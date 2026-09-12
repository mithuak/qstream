//! Specialized time-frequency methods (design Phase 7, Tier C).
//!
//! Stockwell S-transform, reassigned spectrogram, constant-Q transform,
//! fractional Fourier transform, Wigner-Ville distribution, chirp-Z
//! transform, and Kurtogram spectral kurtosis.

use crate::core::fft::{next_pow2, FftPlan};
use crate::core::num::Complex;
use crate::core::ring::RingBuffer;
use crate::signal::spectral::SpectrumResult;

/// Stockwell S-transform.
///
/// A hybrid of the short-time Fourier transform and wavelet transform.
/// Uses a frequency-dependent Gaussian window to achieve better resolution
/// at low frequencies. Returns the time-frequency representation at the
/// window center.
#[derive(Clone, Debug)]
pub struct StockwellTransform {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    nfft: usize,
    fs: f64,
    update_every: usize,
    since: usize,
    plan: FftPlan,
}

impl StockwellTransform {
    pub fn new(window: usize, nfft: usize, update_every: usize) -> Self {
        assert!(window > 0 && nfft > 0);
        let nfft = next_pow2(nfft);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            nfft,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
            plan: FftPlan::new(nfft),
        }
    }

    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
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
        let nfft = self.nfft;
        let bins = nfft / 2 + 1;

        // FFT of the signal
        let mut signal_fft: Vec<Complex> = self
            .data
            .iter()
            .map(|&v| Complex::new(v, 0.0))
            .collect();
        signal_fft.resize(nfft, Complex::new(0.0, 0.0));
        self.plan.fft(&mut signal_fft);

        // For each frequency, multiply by Gaussian-shifted window and IFFT
        let mut power = vec![0.0f64; bins];
        let mut freqs = Vec::with_capacity(bins);

        for k in 0..bins {
            freqs.push(k as f64 * self.fs / nfft as f64);
            if k == 0 {
                power[k] = signal_fft[0].norm_sqr();
                continue;
            }

            // Gaussian window centered at frequency k
            let fk = k as f64;
            let sigma = fk.max(1.0);

            // Shift spectrum by k and multiply by Gaussian
            let mut shifted = vec![Complex::new(0.0, 0.0); nfft];
            for i in 0..nfft {
                let gi = i as f64;
                let window = (-0.5 * (gi * gi) / (sigma * sigma)).exp();
                let src_idx = (i + k) % nfft;
                shifted[i] = signal_fft[src_idx] * Complex::new(window, 0.0);
            }

            // IFFT to get time-domain representation
            self.plan.ifft(&mut shifted);
            // Take the center sample
            power[k] = shifted[n / 2].norm_sqr() / nfft as f64;
        }

        Some(SpectrumResult::from_psd(freqs, power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Reassigned spectrogram.
///
/// Sharpens a standard spectrogram by reassigning each time-frequency
/// bin's energy to the center of gravity of its local energy distribution.
/// Produces a concentrated time-frequency representation.
#[derive(Clone, Debug)]
pub struct ReassignedSpectrogram {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    nfft: usize,
    fs: f64,
    update_every: usize,
    since: usize,
    plan: FftPlan,
}

impl ReassignedSpectrogram {
    pub fn new(window: usize, nfft: usize, update_every: usize) -> Self {
        assert!(window > 0 && nfft > 0);
        let nfft = next_pow2(nfft);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            nfft,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
            plan: FftPlan::new(nfft),
        }
    }

    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
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
        let nfft = self.nfft;
        let bins = nfft / 2 + 1;

        // Compute two STFTs with different windows for reassignment
        let mut s1 = vec![Complex::new(0.0, 0.0); nfft];
        let mut s2 = vec![Complex::new(0.0, 0.0); nfft];

        for i in 0..n {
            let w1 = hann_at(i, n);
            let w2 = if i + 1 < n { hann_at(i + 1, n) } else { 0.0 };
            s1[i] = Complex::new(self.data[i] * w1, 0.0);
            s2[i] = Complex::new(self.data[i] * w2, 0.0);
        }

        self.plan.fft(&mut s1);
        self.plan.fft(&mut s2);

        // Reassignment: shift energy based on phase difference
        let mut reassigned = vec![0.0f64; bins];
        let mut freqs = Vec::with_capacity(bins);

        for k in 0..bins {
            freqs.push(k as f64 * self.fs / nfft as f64);

            let power = s1[k].norm_sqr();
            // Phase gradient for frequency reassignment
            let phase1 = s1[k].arg();
            let phase2 = s2[k].arg();
            let mut dp = phase2 - phase1;
            while dp > std::f64::consts::PI {
                dp -= 2.0 * std::f64::consts::PI;
            }
            while dp < -std::f64::consts::PI {
                dp += 2.0 * std::f64::consts::PI;
            }
            let corrected_freq = (k as f64 + dp * nfft as f64 / (2.0 * std::f64::consts::PI * n as f64)).round() as isize;

            if corrected_freq >= 0 && corrected_freq < bins as isize {
                reassigned[corrected_freq as usize] += power;
            } else {
                reassigned[k] += power;
            }
        }

        Some(SpectrumResult::from_psd(freqs, reassigned))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

fn hann_at(i: usize, n: usize) -> f64 {
    if n <= 1 {
        return 1.0;
    }
    0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / (n - 1) as f64).cos()
}

/// Constant-Q Transform (CQT).
///
/// A time-frequency representation with geometrically-spaced frequency bins,
/// giving constant ratio of center frequency to resolution (Q). Musical and
/// perceptual applications benefit from this logarithmic frequency scale.
#[derive(Clone, Debug)]
pub struct ConstantQTransform {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    n_bins: usize,
    f_min: f64,
    f_max: f64,
    fs: f64,
    update_every: usize,
    since: usize,
    // Precomputed kernels
    kernels: Vec<Vec<Complex>>,
    kernel_len: usize,
}

impl ConstantQTransform {
    pub fn new(window: usize, n_bins: usize, f_min: f64, f_max: f64, update_every: usize) -> Self {
        assert!(window > 0 && n_bins > 0 && f_min > 0.0 && f_max > f_min);
        let fs = 1.0;
        let q = 1.0 / (2.0f64.powf(1.0 / n_bins as f64) - 1.0);
        let mut kernels = Vec::with_capacity(n_bins);
        let mut kernel_len = window;

        for b in 0..n_bins {
            let fb = f_min * 2.0f64.powf(b as f64 / n_bins as f64);
            let kernel_size = (q * fs / fb).round() as usize;
            let kl = kernel_size.max(2).min(window);
            kernel_len = kernel_len.max(kl);

            let mut kernel = Vec::with_capacity(kl);
            for i in 0..kl {
                let t = (i as f64 - kl as f64 / 2.0) / kl as f64;
                let envelope = (-0.5 * (t * q).powi(2)).exp();
                let phase = 2.0 * std::f64::consts::PI * fb * i as f64 / fs;
                kernel.push(Complex::new(
                    envelope * phase.cos() / kl as f64,
                    envelope * phase.sin() / kl as f64,
                ));
            }
            kernels.push(kernel);
        }

        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            n_bins,
            f_min,
            f_max,
            fs,
            update_every: update_every.max(1),
            since: 0,
            kernels,
            kernel_len,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
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
        let mut power = vec![0.0f64; self.n_bins];
        let mut freqs = Vec::with_capacity(self.n_bins);

        for (b, kernel) in self.kernels.iter().enumerate() {
            let fb = self.f_min * 2.0f64.powf(b as f64 / self.n_bins as f64);
            freqs.push(fb * self.fs);

            let kl = kernel.len();
            let mut sum = Complex::new(0.0, 0.0);
            for i in 0..kl.min(n) {
                let s = Complex::new(self.data[i], 0.0);
                sum = sum + s * kernel[i];
            }
            power[b] = sum.norm_sqr();
        }

        Some(SpectrumResult::from_psd(freqs, power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Fractional Fourier Transform (FrFT).
///
/// Generalization of the Fourier transform to arbitrary rotation angles in
/// the time-frequency plane. At angle=pi/2 it equals the standard FFT,
/// at angle=0 it's the identity. Useful for analyzing chirp signals.
#[derive(Clone, Debug)]
pub struct FractionalFourierTransform {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    angle: f64,
    nfft: usize,
    fs: f64,
    update_every: usize,
    since: usize,
}

impl FractionalFourierTransform {
    pub fn new(window: usize, angle: f64, update_every: usize) -> Self {
        assert!(window > 0);
        let nfft = next_pow2(window);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            angle,
            nfft,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
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
        let nfft = self.nfft;
        let a = self.angle;
        let alpha = a * std::f64::consts::PI / 2.0;

        // Simplified FrFT via chirp multiplication - convolution - chirp multiplication
        let cot_alpha = if alpha.abs() < 1e-10 {
            1e10 // near-identity
        } else {
            1.0 / alpha.tan()
        };
        let csc_alpha = if alpha.abs() < 1e-10 {
            1.0
        } else {
            1.0 / alpha.sin()
        };

        // Pre-chirp multiplication
        let mut chirped = vec![Complex::new(0.0, 0.0); nfft];
        for i in 0..n {
            let t = i as f64 - n as f64 / 2.0;
            let phase = std::f64::consts::PI * t * t * cot_alpha / n as f64;
            chirped[i] = Complex::new(self.data[i] * phase.cos(), self.data[i] * phase.sin());
        }

        // FFT
        let plan = FftPlan::new(nfft);
        plan.fft(&mut chirped);

        // Chirp convolution (approximated as multiplication by chirp in freq domain)
        for k in 0..nfft {
            let f = k as f64 - nfft as f64 / 2.0;
            let phase = std::f64::consts::PI * f * f * cot_alpha / n as f64;
            let chirp = Complex::new(phase.cos(), phase.sin());
            chirped[k] = chirped[k] * chirp;
        }

        // IFFT
        plan.ifft(&mut chirped);

        // Post-chirp multiplication
        let scale = (-alpha).exp() * csc_alpha.sqrt() / nfft as f64;
        for i in 0..n {
            let t = i as f64 - n as f64 / 2.0;
            let phase = std::f64::consts::PI * t * t * cot_alpha / n as f64;
            let chirp = Complex::new(phase.cos() * scale, phase.sin() * scale);
            chirped[i] = chirped[i] * chirp;
        }

        // Extract magnitude spectrum
        let bins = nfft / 2 + 1;
        let mut power = Vec::with_capacity(bins);
        let mut freqs = Vec::with_capacity(bins);

        for k in 0..bins {
            freqs.push(k as f64 * self.fs / nfft as f64);
            power.push(chirped[k].norm_sqr());
        }

        Some(SpectrumResult::from_psd(freqs, power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Wigner-Ville Distribution.
///
/// A quadratic time-frequency distribution with high resolution but
/// cross-term interference for multi-component signals. Computed as the
/// Fourier transform of the instantaneous autocorrelation.
#[derive(Clone, Debug)]
pub struct WignerVilleDistribution {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    nfft: usize,
    fs: f64,
    update_every: usize,
    since: usize,
}

impl WignerVilleDistribution {
    pub fn new(window: usize, nfft: usize, update_every: usize) -> Self {
        assert!(window > 0 && nfft > 0);
        let nfft = next_pow2(nfft);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            nfft,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
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
        let nfft = self.nfft;
        let bins = nfft / 2 + 1;

        // Wigner-Ville: W(t, f) = sum_{tau} x(t+tau/2) x*(t-tau/2) e^{-j2pi f tau}
        // Evaluate at the center of the window
        let center = n / 2;
        let mut power = vec![0.0f64; bins];
        let mut freqs = Vec::with_capacity(bins);

        for k in 0..bins {
            freqs.push(k as f64 * self.fs / nfft as f64);
            let f = k as f64 / nfft as f64;
            let mut w_real = 0.0f64;
            let mut w_imag = 0.0f64;

            for tau in 0..n {
                let t_plus = center as isize + tau as isize / 2;
                let t_minus = center as isize - tau as isize / 2;
                if t_plus >= 0 && t_plus < n as isize && t_minus >= 0 && t_minus < n as isize {
                    let xp = self.data[t_plus as usize];
                    let xm = self.data[t_minus as usize];
                    let phase = -2.0 * std::f64::consts::PI * f * tau as f64 / 2.0;
                    w_real += xp * xm * phase.cos();
                    w_imag += xp * xm * phase.sin();
                }
            }
            power[k] = (w_real * w_real + w_imag * w_imag).sqrt() / n as f64;
        }

        Some(SpectrumResult::from_psd(freqs, power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Chirp-Z Transform (zoom FFT).
///
/// Evaluates the Z-transform along a spiral contour in the Z-plane, allowing
/// zoomed-in spectral analysis over a narrow frequency band with arbitrary
/// resolution.
#[derive(Clone, Debug)]
pub struct ChirpZTransform {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    f_start: f64,
    f_end: f64,
    n_points: usize,
    fs: f64,
    update_every: usize,
    since: usize,
}

impl ChirpZTransform {
    pub fn new(
        window: usize,
        f_start: f64,
        f_end: f64,
        n_points: usize,
        update_every: usize,
    ) -> Self {
        assert!(window > 0 && n_points > 0 && f_start < f_end);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            f_start,
            f_end,
            n_points,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
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
        let m = self.n_points;

        // Z-transform: X(z_k) = sum_{n=0}^{N-1} x[n] z_k^{-n}
        // where z_k = A W^{-k}, A = e^{j 2pi f_start}, W = e^{-j 2pi (f_end-f_start)/(M-1)}
        let a = Complex::new(
            (2.0 * std::f64::consts::PI * self.f_start / self.fs).cos(),
            (2.0 * std::f64::consts::PI * self.f_start / self.fs).sin(),
        );
        let w_angle = -2.0 * std::f64::consts::PI * (self.f_end - self.f_start) / (self.fs * (m - 1).max(1) as f64);
        let w = Complex::new(w_angle.cos(), w_angle.sin());

        let mut power = Vec::with_capacity(m);
        let mut freqs = Vec::with_capacity(m);

        for k in 0..m {
            let f = self.f_start + k as f64 * (self.f_end - self.f_start) / (m - 1).max(1) as f64;
            freqs.push(f * self.fs);

            // z_k = A * W^{-k}
            let z_k = a * w.powi(-(k as i32));

            // Compute X(z_k)
            let mut sum = Complex::new(0.0, 0.0);
            let mut z_inv_n = Complex::new(1.0, 0.0);
            let z_inv = Complex::new(1.0, 0.0) / z_k;
            for i in 0..n {
                sum = sum + Complex::new(self.data[i], 0.0) * z_inv_n;
                z_inv_n = z_inv_n * z_inv;
            }
            power.push(sum.norm_sqr());
        }

        Some(SpectrumResult::from_psd(freqs, power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Kurtogram: spectral kurtosis across frequency bands.
///
/// Computes the spectral kurtosis at each frequency band to detect
/// non-Gaussian (impulsive) behavior. High spectral kurtosis indicates
/// frequency bands containing transients or impulses.
#[derive(Clone, Debug)]
pub struct KurtogramAnalysis {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    nfft: usize,
    fs: f64,
    update_every: usize,
    since: usize,
    plan: FftPlan,
}

impl KurtogramAnalysis {
    pub fn new(window: usize, nfft: usize, update_every: usize) -> Self {
        assert!(window > 0 && nfft > 0);
        let nfft = next_pow2(nfft);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            nfft,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
            plan: FftPlan::new(nfft),
        }
    }

    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
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
        let nfft = self.nfft;
        let bins = nfft / 2 + 1;

        // Compute STFT with overlapping segments for kurtosis estimation
        let n_segments = 4;
        let hop = n / n_segments;

        // Collect magnitude spectra at each frequency
        let mut segment_magnitudes: Vec<Vec<f64>> = vec![vec![0.0f64; bins]; n_segments];

        for seg in 0..n_segments {
            let start = seg * hop;
            let end = (start + n).min(n);
            let len = end - start;

            let mut frame = vec![Complex::new(0.0, 0.0); nfft];
            for i in 0..len {
                let w = hann_at(i, len);
                frame[i] = Complex::new(self.data[start + i] * w, 0.0);
            }
            self.plan.fft(&mut frame);

            for k in 0..bins {
                segment_magnitudes[seg][k] = frame[k].abs();
            }
        }

        // Spectral kurtosis at each frequency
        let mut power = vec![0.0f64; bins];
        let mut freqs = Vec::with_capacity(bins);

        for k in 0..bins {
            freqs.push(k as f64 * self.fs / nfft as f64);

            // Kurtosis = E[|X|^4] / (E[|X|^2])^2 - 3 (excess)
            let mut m2 = 0.0f64;
            let mut m4 = 0.0f64;
            for seg in 0..n_segments {
                let m = segment_magnitudes[seg][k];
                m2 += m * m;
                m4 += m * m * m * m;
            }
            m2 /= n_segments as f64;
            m4 /= n_segments as f64;

            power[k] = if m2 > 1e-30 {
                m4 / (m2 * m2) - 3.0
            } else {
                0.0
            };
        }

        Some(SpectrumResult::from_psd(freqs, power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

// Helper trait for Complex power operations
trait ComplexExt {
    fn powi(&self, n: i32) -> Complex;
}

impl ComplexExt for Complex {
    fn powi(&self, n: i32) -> Complex {
        if n == 0 {
            return Complex::new(1.0, 0.0);
        }
        let mut result = *self;
        for _ in 1..n.abs() {
            result = result * *self;
        }
        if n < 0 {
            Complex::new(1.0, 0.0) / result
        } else {
            result
        }
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
    fn stockwell_runs() {
        let mut st = StockwellTransform::new(64, 64, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = st.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.power.iter().all(|p| p.is_finite()));
    }

    #[test]
    fn reassigned_spectrogram_runs() {
        let mut rs = ReassignedSpectrogram::new(64, 64, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = rs.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.power.iter().all(|p| p.is_finite()));
    }

    #[test]
    fn constant_q_runs() {
        let mut cqt = ConstantQTransform::new(64, 24, 0.01, 0.4, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = cqt.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.power.iter().all(|p| p.is_finite()));
    }

    #[test]
    fn fractional_fourier_runs() {
        let mut frft = FractionalFourierTransform::new(64, std::f64::consts::PI / 4.0, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = frft.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.power.iter().all(|p| p.is_finite()));
    }

    #[test]
    fn wigner_ville_runs() {
        let mut wv = WignerVilleDistribution::new(64, 64, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = wv.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.power.iter().all(|p| p.is_finite()));
    }

    #[test]
    fn chirp_z_runs() {
        let mut czt = ChirpZTransform::new(64, 0.1, 0.3, 64, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = czt.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.power.iter().all(|p| p.is_finite()));
    }

    #[test]
    fn kurtogram_runs() {
        let mut kg = KurtogramAnalysis::new(64, 64, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = kg.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.power.iter().all(|p| p.is_finite()));
    }
}
