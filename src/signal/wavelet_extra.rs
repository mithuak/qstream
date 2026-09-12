//! Specialized/experimental wavelet transforms (design Phase 7, Tier C).
//!
//! Dual-tree complex wavelet transform, empirical wavelet transform (Gilles),
//! tunable-Q wavelet transform, SureShrink denoising, stationary (undecimated)
//! denoising, cross-wavelet transform, wavelet phase synchrony, and wavelet
//! regression. These build on the core DWT primitives.

use crate::core::ring::RingBuffer;
use crate::signal::wavelet::WaveletResult;

/// Dual-Tree Complex Wavelet Transform (DTCWT).
///
/// Uses two parallel DWT trees with specially-designed filters to achieve
/// approximate shift-invariance and directional selectivity. Returns complex
/// coefficients (real and imaginary parts interleaved).
#[derive(Clone, Debug)]
pub struct DualTreeCwt {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    levels: usize,
    update_every: usize,
    since: usize,
    // Two sets of filter coefficients (approximations of Hilbert pair pair)
    // Tree A: [1, -1] highpass, [1, 1] lowpass (Haar-like)
    // Tree B: shifted versions
}

impl DualTreeCwt {
    pub fn new(window: usize, levels: usize, update_every: usize) -> Self {
        assert!(window > 0 && levels > 0 && levels <= 8);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            levels,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<WaveletResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();
        let mut real_a = self.data.clone();
        let mut imag_a = self.data.clone();

        // Two trees: Tree A uses standard Haar, Tree B uses shifted Haar
        let mut real_coeffs = Vec::new();
        let mut imag_coeffs = Vec::new();
        let mut scales = Vec::new();

        for level in 0..self.levels {
            let len = real_a.len();
            if len < 2 {
                break;
            }
            let half = len / 2;

            // Tree A: standard Haar
            let mut a_r = vec![0.0f64; half];
            let mut d_r = vec![0.0f64; half];
            for i in 0..half {
                a_r[i] = (real_a[2 * i] + real_a[2 * i + 1]) / 2.0f64.sqrt();
                d_r[i] = (real_a[2 * i] - real_a[2 * i + 1]) / 2.0f64.sqrt();
            }

            // Tree B: shifted Haar (offset by 1 sample)
            let mut a_i = vec![0.0f64; half];
            let mut d_i = vec![0.0f64; half];
            for i in 0..half {
                let i0 = (2 * i + 1).min(len - 1);
                let i1 = (2 * i + 2).min(len - 1);
                a_i[i] = (imag_a[i0] + imag_a[i1]) / 2.0f64.sqrt();
                d_i[i] = (imag_a[i0] - imag_a[i1]) / 2.0f64.sqrt();
            }

            // Store detail coefficients as complex: real = d_r, imag = d_i
            for i in 0..half {
                real_coeffs.push(d_r[i]);
                imag_coeffs.push(d_i[i]);
            }
            scales.push((1 << (level + 1)) as f64);

            real_a = a_r;
            imag_a = a_i;
        }

        // Append approximation
        for i in 0..real_a.len() {
            real_coeffs.push(real_a[i]);
            imag_coeffs.push(imag_a[i]);
        }

        // Interleave real and imaginary parts
        let mut coefficients = Vec::with_capacity(real_coeffs.len() * 2);
        for i in 0..real_coeffs.len() {
            coefficients.push(real_coeffs[i]);
            coefficients.push(imag_coeffs[i]);
        }

        Some(WaveletResult {
            coefficients,
            scales: Some(scales),
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Empirical Wavelet Transform (EWT, Gilles 2013).
///
/// Builds an adaptive wavelet filter bank by detecting boundaries in the
/// Fourier spectrum of the signal, then applies the empirical wavelets to
/// extract AM-FM modes.
#[derive(Clone, Debug)]
pub struct EmpiricalWaveletTransform {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    n_modes: usize,
    update_every: usize,
    since: usize,
}

impl EmpiricalWaveletTransform {
    pub fn new(window: usize, n_modes: usize, update_every: usize) -> Self {
        assert!(window > 0 && n_modes > 0 && n_modes <= 16);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            n_modes,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<WaveletResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();
        let nfft = n.next_power_of_two();

        // Compute FFT magnitude spectrum
        let mut spectrum = vec![0.0f64; nfft / 2 + 1];
        for k in 0..spectrum.len() {
            let w = 2.0 * std::f64::consts::PI * k as f64 / nfft as f64;
            let mut re = 0.0f64;
            let mut im = 0.0f64;
            for i in 0..n {
                re += self.data[i] * (w * i as f64).cos();
                im -= self.data[i] * (w * i as f64).sin();
            }
            spectrum[k] = (re * re + im * im).sqrt();
        }

        // Find boundaries: local minima separating peaks
        let boundaries = detect_boundaries(&spectrum, self.n_modes);

        // Build empirical wavelet filters and apply
        let mut coefficients = Vec::new();
        let mut scales = Vec::new();

        for (start, end) in boundaries.iter().zip(boundaries.iter().skip(1)) {
            let center = (start + end) / 2;
            let bandwidth = (end - start).max(1);

            // Apply bandpass filter in frequency domain
            let mut mode = vec![0.0f64; n];
            for i in 0..n {
                let mut val = 0.0f64;
                for k in *start..*end {
                    let w = 2.0 * std::f64::consts::PI * k as f64 / nfft as f64;
                    let offset = (k as isize - center as isize).unsigned_abs();
                    let transfer = empirical_wavelet_transfer(offset, bandwidth);
                    val += spectrum[k] * transfer * (w * i as f64).cos();
                }
                mode[i] = val / nfft as f64;
            }
            coefficients.extend_from_slice(&mode);
            scales.push(bandwidth as f64);
        }

        Some(WaveletResult {
            coefficients,
            scales: Some(scales),
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Detect spectrum boundaries by finding local minima.
fn detect_boundaries(spectrum: &[f64], n_modes: usize) -> Vec<usize> {
    let n = spectrum.len();
    if n < 3 {
        return vec![0, n];
    }

    // Find local minima
    let mut minima: Vec<(usize, f64)> = Vec::new();
    for i in 1..n - 1 {
        if spectrum[i] <= spectrum[i - 1] && spectrum[i] <= spectrum[i + 1] {
            minima.push((i, spectrum[i]));
        }
    }

    // Sort by value (smallest first = deepest valleys)
    minima.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Take the n_modes-1 deepest minima as boundaries
    let mut boundaries: Vec<usize> = minima
        .iter()
        .take(n_modes - 1)
        .map(|(i, _)| *i)
        .collect();
    boundaries.sort_unstable();

    // Always include 0 and n
    let mut result = vec![0];
    result.extend(boundaries);
    result.push(n);
    result
}

/// Empirical wavelet transfer function (Meyer-type scaling).
fn empirical_wavelet_transfer(freq_offset: usize, bandwidth: usize) -> f64 {
    let gamma = 0.1; // transition ratio
    let bw = bandwidth.max(1) as f64;
    let f = freq_offset as f64;
    let transition = gamma * bw;

    if f.abs() <= bw / 2.0 - transition {
        1.0
    } else if f.abs() <= bw / 2.0 + transition {
        let t = (bw / 2.0 + transition - f.abs()) / (2.0 * transition);
        (0.5 + 0.5 * (std::f64::consts::PI * t).cos()).sqrt()
    } else {
        0.0
    }
}

/// Tunable-Q Wavelet Transform (TQWT).
///
/// Wavelet transform with a tunable Q-factor (ratio of center frequency to
/// bandwidth). High Q for oscillatory signals, low Q for transient signals.
/// Uses an iterative two-channel filter bank.
#[derive(Clone, Debug)]
pub struct TunableQWavelet {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    levels: usize,
    q_factor: f64,
    redundancy: f64,
    update_every: usize,
    since: usize,
}

impl TunableQWavelet {
    pub fn new(window: usize, levels: usize, q_factor: f64, update_every: usize) -> Self {
        assert!(window > 0 && levels > 0 && levels <= 12);
        assert!(q_factor >= 1.0);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            levels,
            q_factor,
            redundancy: 3.0, // oversampling factor
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<WaveletResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();
        let alpha = 1.0 - (self.q_factor - 1.0) / (self.q_factor + 1.0).max(1.0);
        let beta = 2.0 / (self.q_factor + 1.0).max(1.0);

        let mut coefficients = Vec::new();
        let mut scales = Vec::new();
        let mut current = self.data.clone();

        for level in 0..self.levels {
            let len = current.len();
            if len < 2 {
                break;
            }
            let half = (len as f64 * alpha).round() as usize;
            let half = half.max(1).min(len - 1);

            // Lowpass and highpass subband coding
            let mut low = vec![0.0f64; half];
            let mut high = vec![0.0f64; len - half];

            for i in 0..half {
                let src = (i as f64 / alpha).round() as usize;
                low[i] = current[src.min(len - 1)] * beta;
            }
            for i  in 0..len - half {
                let src = (half as f64 + i as f64 / (1.0 - alpha)).round() as usize;
                high[i] = current[src.min(len - 1)] * (1.0 - beta);
            }

            coefficients.extend_from_slice(&high);
            scales.push((1 << (level + 1)) as f64);
            current = low;
        }

        // Append final lowpass
        coefficients.extend_from_slice(&current);

        Some(WaveletResult {
            coefficients,
            scales: Some(scales),
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// SureShrink wavelet denoising.
///
/// Applies a universal threshold based on Stein's Unbiased Risk Estimate
/// (SURE) to wavelet coefficients. Uses soft thresholding to suppress noise
/// while preserving signal features.
#[derive(Clone, Debug)]
pub struct SureShrinkDenoise {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    levels: usize,
    update_every: usize,
    since: usize,
}

impl SureShrinkDenoise {
    pub fn new(window: usize, levels: usize, update_every: usize) -> Self {
        assert!(window > 0 && levels > 0 && levels <= 8);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            levels,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<WaveletResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();

        // Decompose using Haar DWT
        let mut detail_coeffs: Vec<Vec<f64>> = Vec::new();
        let mut approx = self.data.clone();

        for _ in 0..self.levels {
            if approx.len() < 2 {
                break;
            }
            let half = approx.len() / 2;
            let mut low = vec![0.0f64; half];
            let mut high = vec![0.0f64; half];
            for i in 0..half {
                low[i] = (approx[2 * i] + approx[2 * i + 1]) / 2.0f64.sqrt();
                high[i] = (approx[2 * i] - approx[2 * i + 1]) / 2.0f64.sqrt();
            }
            detail_coeffs.push(high);
            approx = low;
        }

        // Estimate noise level from finest detail using MAD
        if let Some(finest) = detail_coeffs.first() {
            let sigma = mad(finest) / 0.6745;
            let threshold = sigma * (2.0 * n as f64).ln().sqrt();

            // Apply soft thresholding to all detail coefficients
            for detail in detail_coeffs.iter_mut() {
                for c in detail.iter_mut() {
                    let abs_c = c.abs();
                    if abs_c <= threshold {
                        *c = 0.0;
                    } else {
                        *c = c.signum() * (abs_c - threshold);
                    }
                }
            }
        }

        // Reconstruct
        for detail in detail_coeffs.iter().rev() {
            let mut new_approx = vec![0.0f64; detail.len() * 2];
            for i in 0..detail.len() {
                new_approx[2 * i] = (approx[i] + detail[i]) / 2.0f64.sqrt();
                new_approx[2 * i + 1] = (approx[i] - detail[i]) / 2.0f64.sqrt();
            }
            approx = new_approx;
        }

        Some(WaveletResult {
            coefficients: approx,
            scales: None,
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Median Absolute Deviation (robust scale estimator).
fn mad(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[sorted.len() / 2];
    let mut abs_devs: Vec<f64> = data.iter().map(|x| (x - median).abs()).collect();
    abs_devs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    abs_devs[abs_devs.len() / 2]
}

/// Stationary (Undecimated) Wavelet Denoising.
///
/// Also known as the `a trous` algorithm. Applies the wavelet transform
/// without downsampling, making it shift-invariant. Denoises by
/// thresholding the wavelet coefficients at each scale.
#[derive(Clone, Debug)]
pub struct StationaryWaveletDenoise {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    levels: usize,
    update_every: usize,
    since: usize,
}

impl StationaryWaveletDenoise {
    pub fn new(window: usize, levels: usize, update_every: usize) -> Self {
        assert!(window > 0 && levels > 0 && levels <= 8);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            levels,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<WaveletResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();

        // Stationary wavelet transform (a trous) with Haar
        let mut details: Vec<Vec<f64>> = Vec::new();
        let mut approx = self.data.clone();

        for level in 0..self.levels {
            let step = 1 << level;
            let mut detail = vec![0.0f64; n];
            let mut new_approx = vec![0.0f64; n];

            for i in 0..n {
                let left = approx[i];
                let right = approx[(i + step) % n];
                new_approx[i] = (left + right) / 2.0;
                detail[i] = (left - right) / 2.0;
            }

            details.push(detail);
            approx = new_approx;
        }

        // Estimate noise and threshold
        if let Some(finest) = details.first() {
            let sigma = mad(finest) / 0.6745;
            let threshold = sigma * (2.0 * n as f64).ln().sqrt();

            for detail in details.iter_mut() {
                for c in detail.iter_mut() {
                    let abs_c = c.abs();
                    if abs_c <= threshold {
                        *c = 0.0;
                    } else {
                        *c = c.signum() * (abs_c - threshold);
                    }
                }
            }
        }

        // Reconstruct
        let mut result = approx;
        for detail in details.iter() {
            for i in 0..n {
                result[i] += detail[i];
            }
        }

        Some(WaveletResult {
            coefficients: result,
            scales: None,
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Cross-Wavelet Transform (XWT).
///
/// Computes the wavelet transform of two signals and their cross-wavelet
/// spectrum, revealing time-frequency regions of high common power and
/// consistent phase relationship.
#[derive(Clone, Debug)]
pub struct CrossWaveletTransform {
    bufx: RingBuffer<f64>,
    bufy: RingBuffer<f64>,
    dx: Vec<f64>,
    dy: Vec<f64>,
    levels: usize,
    update_every: usize,
    since: usize,
}

impl CrossWaveletTransform {
    pub fn new(window: usize, levels: usize, update_every: usize) -> Self {
        assert!(window > 0 && levels > 0 && levels <= 8);
        Self {
            bufx: RingBuffer::new(window, 0.0),
            bufy: RingBuffer::new(window, 0.0),
            dx: Vec::with_capacity(window),
            dy: Vec::with_capacity(window),
            levels,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64, y: f64) -> Option<WaveletResult> {
        self.bufx.push(x);
        self.bufy.push(y);
        if !self.bufx.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.bufx.fill_vec(&mut self.dx);
        self.bufy.fill_vec(&mut self.dy);

        let n = self.dx.len();

        // Compute DWT of both signals
        let mut x_details = Vec::new();
        let mut y_details = Vec::new();
        let mut x_approx = self.dx.clone();
        let mut y_approx = self.dy.clone();

        for _ in 0..self.levels {
            if x_approx.len() < 2 {
                break;
            }
            let half = x_approx.len() / 2;
            let mut x_low = vec![0.0f64; half];
            let mut x_high = vec![0.0f64; half];
            let mut y_low = vec![0.0f64; half];
            let mut y_high = vec![0.0f64; half];
            for i in 0..half {
                x_low[i] = (x_approx[2 * i] + x_approx[2 * i + 1]) / 2.0f64.sqrt();
                x_high[i] = (x_approx[2 * i] - x_approx[2 * i + 1]) / 2.0f64.sqrt();
                y_low[i] = (y_approx[2 * i] + y_approx[2 * i + 1]) / 2.0f64.sqrt();
                y_high[i] = (y_approx[2 * i] - y_approx[2 * i + 1]) / 2.0f64.sqrt();
            }
            x_details.push(x_high);
            y_details.push(y_high);
            x_approx = x_low;
            y_approx = y_low;
        }

        // Cross-wavelet coefficients: |W_x * conj(W_y)|
        let mut coefficients = Vec::new();
        let mut scales = Vec::new();

        for (xd, yd) in x_details.iter().zip(y_details.iter()) {
            for i in 0..xd.len() {
                // Cross-wavelet power
                coefficients.push((xd[i] * xd[i] + yd[i] * yd[i]).sqrt());
            }
            scales.push((1 << (scales.len() + 1)) as f64);
        }

        Some(WaveletResult {
            coefficients,
            scales: Some(scales),
        })
    }

    pub fn reset(&mut self) {
        self.bufx.clear(0.0);
        self.bufy.clear(0.0);
        self.dx.clear();
        self.dy.clear();
        self.since = 0;
    }
}

/// Wavelet Phase Synchrony.
///
/// Measures the phase synchronization between two signals in the wavelet
/// domain. Computes the phase locking value (PLV) across scales: a value
/// near 1 indicates consistent phase difference (synchronization), near 0
/// indicates no phase relationship.
#[derive(Clone, Debug)]
pub struct WaveletPhaseSynchrony {
    bufx: RingBuffer<f64>,
    bufy: RingBuffer<f64>,
    dx: Vec<f64>,
    dy: Vec<f64>,
    levels: usize,
    update_every: usize,
    since: usize,
}

impl WaveletPhaseSynchrony {
    pub fn new(window: usize, levels: usize, update_every: usize) -> Self {
        assert!(window > 0 && levels > 0 && levels <= 8);
        Self {
            bufx: RingBuffer::new(window, 0.0),
            bufy: RingBuffer::new(window, 0.0),
            dx: Vec::with_capacity(window),
            dy: Vec::with_capacity(window),
            levels,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64, y: f64) -> Option<WaveletResult> {
        self.bufx.push(x);
        self.bufy.push(y);
        if !self.bufx.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.bufx.fill_vec(&mut self.dx);
        self.bufy.fill_vec(&mut self.dy);

        let n = self.dx.len();

        // Compute analytic signals via Hilbert-like approach (using DWT)
        let mut x_details = Vec::new();
        let mut y_details = Vec::new();
        let mut x_approx = self.dx.clone();
        let mut y_approx = self.dy.clone();

        for _ in 0..self.levels {
            if x_approx.len() < 2 {
                break;
            }
            let half = x_approx.len() / 2;
            let mut x_low = vec![0.0f64; half];
            let mut x_high = vec![0.0f64; half];
            let mut y_low = vec![0.0f64; half];
            let mut y_high = vec![0.0f64; half];
            for i in 0..half {
                x_low[i] = (x_approx[2 * i] + x_approx[2 * i + 1]) / 2.0f64.sqrt();
                x_high[i] = (x_approx[2 * i] - x_approx[2 * i + 1]) / 2.0f64.sqrt();
                y_low[i] = (y_approx[2 * i] + y_approx[2 * i + 1]) / 2.0f64.sqrt();
                y_high[i] = (y_approx[2 * i] - y_approx[2 * i + 1]) / 2.0f64.sqrt();
            }
            x_details.push(x_high);
            y_details.push(y_high);
            x_approx = x_low;
            y_approx = y_low;
        }

        // Phase locking value at each scale
        let mut coefficients = Vec::new();
        let mut scales = Vec::new();

        for (level, (xd, yd)) in x_details.iter().zip(y_details.iter()).enumerate() {
            let mut sum_cos = 0.0f64;
            let mut count = 0usize;

            for i in 0..xd.len() {
                let phase_x = yd[i].atan2(xd[i]);
                let phase_y = xd[i].atan2(yd[i]);
                let phase_diff = phase_y - phase_x;
                sum_cos += phase_diff.cos();
                count += 1;
            }

            let plv = if count > 0 {
                (sum_cos / count as f64).abs()
            } else {
                0.0
            };
            coefficients.push(plv);
            scales.push((1 << (level + 1)) as f64);
        }

        Some(WaveletResult {
            coefficients,
            scales: Some(scales),
        })
    }

    pub fn reset(&mut self) {
        self.bufx.clear(0.0);
        self.bufy.clear(0.0);
        self.dx.clear();
        self.dy.clear();
        self.since = 0;
    }
}

/// Wavelet Regression.
///
/// Denoises a signal using wavelet shrinkage and returns the regression
/// (denoised) estimate. Uses a level-dependent threshold based on the
/// variance of coefficients at each scale.
#[derive(Clone, Debug)]
pub struct WaveletRegression {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    levels: usize,
    update_every: usize,
    since: usize,
}

impl WaveletRegression {
    pub fn new(window: usize, levels: usize, update_every: usize) -> Self {
        assert!(window > 0 && levels > 0 && levels <= 8);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            levels,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<WaveletResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();

        // Decompose
        let mut details: Vec<Vec<f64>> = Vec::new();
        let mut approx = self.data.clone();

        for _ in 0..self.levels {
            if approx.len() < 2 {
                break;
            }
            let half = approx.len() / 2;
            let mut low = vec![0.0f64; half];
            let mut high = vec![0.0f64; half];
            for i in 0..half {
                low[i] = (approx[2 * i] + approx[2 * i + 1]) / 2.0f64.sqrt();
                high[i] = (approx[2 * i] - approx[2 * i + 1]) / 2.0f64.sqrt();
            }
            details.push(high);
            approx = low;
        }

        // Level-dependent thresholding
        for (level, detail) in details.iter_mut().enumerate() {
            let sigma = mad(detail) / 0.6745;
            let threshold = sigma * (2.0 * (detail.len() as f64).ln()).sqrt()
                / (1.0 + level as f64).sqrt();

            for c in detail.iter_mut() {
                let abs_c = c.abs();
                if abs_c <= threshold {
                    *c = 0.0;
                } else {
                    *c = c.signum() * (abs_c - threshold);
                }
            }
        }

        // Reconstruct
        for detail in details.iter().rev() {
            let mut new_approx = vec![0.0f64; detail.len() * 2];
            for i in 0..detail.len() {
                new_approx[2 * i] = (approx[i] + detail[i]) / 2.0f64.sqrt();
                new_approx[2 * i + 1] = (approx[i] - detail[i]) / 2.0f64.sqrt();
            }
            approx = new_approx;
        }

        Some(WaveletResult {
            coefficients: approx,
            scales: None,
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(n: usize, freq: f64) -> Vec<f64> {
        (0..n)
            .map(|i| (2.0 * std::f64::consts::PI * freq * i as f64).sin())
            .collect()
    }

    #[test]
    fn dual_tree_cwt_runs() {
        let mut dtcwt = DualTreeCwt::new(64, 3, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = dtcwt.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.coefficients.len() > 0);
    }

    #[test]
    fn empirical_wavelet_runs() {
        let mut ewt = EmpiricalWaveletTransform::new(64, 3, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = ewt.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.coefficients.len() > 0);
    }

    #[test]
    fn tunable_q_runs() {
        let mut tqwt = TunableQWavelet::new(64, 4, 2.0, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = tqwt.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.coefficients.len() > 0);
    }

    #[test]
    fn sure_shrink_denoises() {
        let mut ss = SureShrinkDenoise::new(64, 3, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = ss.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert_eq!(r.coefficients.len(), 64);
    }

    #[test]
    fn stationary_wavelet_runs() {
        let mut swd = StationaryWaveletDenoise::new(64, 3, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = swd.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert_eq!(r.coefficients.len(), 64);
    }

    #[test]
    fn cross_wavelet_runs() {
        let mut xwt = CrossWaveletTransform::new(64, 3, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = xwt.update(x, x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.coefficients.len() > 0);
    }

    #[test]
    fn wavelet_phase_synchrony_runs() {
        let mut wps = WaveletPhaseSynchrony::new(64, 3, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = wps.update(x, x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        // Identical signals should have high phase synchrony
        assert!(r.coefficients.iter().any(|&v| v > 0.5));
    }

    #[test]
    fn wavelet_regression_runs() {
        let mut wr = WaveletRegression::new(64, 3, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = wr.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert_eq!(r.coefficients.len(), 64);
    }
}
