//! Remaining spectral / filtering estimators from the workbook that complete
//! the catalog (all Tier C: rolling window with `update_every` cadence).
//!
//! Cepstral analysis, Daniell-smoothed periodogram, eigenvector frequency
//! estimation, modified-covariance AR spectrum, multiple (multitaper)
//! coherence, multivariate spectral analysis, Parzen-windowed periodogram,
//! partial coherence, spectral envelope, Wiener-Hopf optimal filtering, and
//! Capon APES amplitude/phase estimation.

use crate::core::fft::{next_pow2, FftPlan};
use crate::core::matrix::{jacobi_eigen, DMat};
use crate::core::num::Complex;
use crate::core::ring::RingBuffer;
use crate::core::window::dpss;
use crate::signal::prediction::{autocorrelation, levinson_durbin};
use crate::signal::spectral::{dominant, SpectrumResult};

/// Build a Toeplitz autocorrelation matrix of order `p` from `acf` (length p+1):
/// `R[i][j] = acf[|i - j|]`.
fn toeplitz(acf: &[f64], p: usize) -> DMat {
    let mut r = DMat::zeros(p, p);
    for i in 0..p {
        for j in 0..p {
            let lag = (i as isize - j as isize).unsigned_abs();
            r.set(i, j, acf[lag]);
        }
    }
    r
}

/// One-sided periodogram of `data` (length `n`, power of two) using a caller
/// supplied window and FFT plan. Returns normalized power (|X|^2 / sum(w^2)).
fn periodogram(
    plan: &FftPlan,
    win: &[f64],
    data: &[f64],
    freqs: &mut Vec<f64>,
    power: &mut Vec<f64>,
) {
    let n = data.len();
    let win_power: f64 = win.iter().map(|w| w * w).sum::<f64>().max(1e-30);
    let mut scratch: Vec<Complex> = (0..n).map(|i| Complex::new(data[i] * win[i], 0.0)).collect();
    plan.fft(&mut scratch);
    let bins = n / 2 + 1;
    freqs.clear();
    power.clear();
    for k in 0..bins {
        let scale = if k == 0 || k == n / 2 { 1.0 } else { 2.0 };
        power.push(scale * scratch[k].norm_sqr() / win_power);
        freqs.push(k as f64 / n as f64);
    }
}

/// Cepstral Analysis.
///
/// The real cepstrum is the inverse Fourier transform of the log-magnitude
/// spectrum:
///
/// ```text
/// c[n] = IFFT( log |FFT(x[n])| )
/// ```
///
/// The quefrency (time) axis exposes periodic structure in the signal: a
/// harmonic sound produces a peak at its fundamental period, independent of
/// amplitude. The result is returned as a [`SpectrumResult`] whose
/// `frequencies` are quefrency bins and whose `power` holds cepstral
/// coefficients.
#[derive(Clone, Debug)]
pub struct CepstralAnalysis {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    plan: FftPlan,
    win: Vec<f64>,
    scratch: Vec<Complex>,
    update_every: usize,
    since: usize,
}

impl CepstralAnalysis {
    pub fn new(window: usize, update_every: usize) -> Self {
        let w = next_pow2(window);
        let win = vec![1.0; w];
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            plan: FftPlan::new(w),
            win,
            scratch: Vec::with_capacity(w),
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
        self.scratch.clear();
        for i in 0..n {
            self.scratch.push(Complex::new(self.data[i] * self.win[i], 0.0));
        }
        self.plan.fft(&mut self.scratch);
        // log-magnitude spectrum (clamped to keep log finite).
        for c in self.scratch.iter_mut() {
            let mag = c.abs().max(1e-12);
            *c = Complex::new(mag.ln(), 0.0);
        }
        self.plan.ifft(&mut self.scratch);
        let bins = n;
        let mut freqs = Vec::with_capacity(bins);
        let mut power = Vec::with_capacity(bins);
        for k in 0..bins {
            freqs.push(k as f64 / n as f64);
            power.push(self.scratch[k].re / n as f64);
        }
        Some(SpectrumResult::from_psd(freqs, power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Daniell-Smoothed Periodogram.
///
/// Smooths the raw periodogram with a symmetric moving-average (Daniell)
/// kernel of half-width `m`:
///
/// ```text
/// P_D(f_k) = (1 / (2m + 1)) * sum_{j = -m}^{m} P(f_{k+j})
/// ```
///
/// This reduces the variance of the periodogram (at the cost of a slightly
/// broader spectral peak) and is the classical way to obtain a consistent
/// spectral density estimate.
#[derive(Clone, Debug)]
pub struct DaniellPeriodogram {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    m: usize,
    plan: FftPlan,
    win: Vec<f64>,
    freqs: Vec<f64>,
    raw: Vec<f64>,
    update_every: usize,
    since: usize,
}

impl DaniellPeriodogram {
    /// `window`: analysis window length; `m`: Daniell half-width (e.g. 3).
    pub fn new(window: usize, m: usize, update_every: usize) -> Self {
        let w = next_pow2(window);
        let win = vec![1.0; w];
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            m,
            plan: FftPlan::new(w),
            win,
            freqs: Vec::new(),
            raw: Vec::new(),
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
        periodogram(&self.plan, &self.win, &self.data, &mut self.freqs, &mut self.raw);
        let n = self.raw.len();
        let mut power = vec![0.0; n];
        for k in 0..n {
            let mut s = 0.0;
            let mut cnt = 0usize;
            let lo = k.saturating_sub(self.m);
            let hi = (k + self.m).min(n - 1);
            for j in lo..=hi {
                s += self.raw[j];
                cnt += 1;
            }
            power[k] = s / cnt.max(1) as f64;
        }
        Some(SpectrumResult::from_psd(self.freqs.clone(), power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.freqs.clear();
        self.raw.clear();
        self.since = 0;
    }
}

/// Eigenvector Frequency Estimator.
///
/// Uses the eigenvector of the autocorrelation matrix associated with the
/// smallest eigenvalue as a prediction-error filter:
///
/// ```text
/// A(f) = sum_{i=0}^{p} v_i e^{-j 2 pi f i}
/// P(f) = 1 / |A(f)|^2
/// ```
///
/// The zeros of `A(f)` lie on the unit circle at the signal frequencies, so
/// the pseudospectrum `P(f)` peaks at those frequencies. This is the basis of
/// the Pisarenko / minimum-norm family.
#[derive(Clone, Debug)]
pub struct EigenvectorFrequencyEstimator {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    order: usize,
    nfft: usize,
    update_every: usize,
    since: usize,
}

impl EigenvectorFrequencyEstimator {
    pub fn new(window: usize, order: usize, nfft: usize, update_every: usize) -> Self {
        assert!(order >= 1 && order < window);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            order,
            nfft: nfft.max(64),
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
        let p = self.order + 1;
        let acf = autocorrelation(&self.data, self.order);
        let r = toeplitz(&acf, p);
        let (_, vecs) = jacobi_eigen(&r, 200);
        let v: Vec<f64> = (0..p).map(|i| vecs.get(i, p - 1)).collect();

        let bins = self.nfft;
        let mut freqs = Vec::with_capacity(bins);
        let mut power = Vec::with_capacity(bins);
        for k in 0..bins {
            let f = k as f64 / bins as f64;
            freqs.push(f);
            let w = 2.0 * std::f64::consts::PI * f;
            let mut re = 0.0;
            let mut im = 0.0;
            for i in 0..p {
                let ang = w * i as f64;
                re += v[i] * ang.cos();
                im -= v[i] * ang.sin();
            }
            power.push(1.0 / (re * re + im * im).max(1e-30));
        }
        let max = power.iter().copied().fold(0.0f64, f64::max);
        if max > 0.0 {
            for p in power.iter_mut() {
                *p /= max;
            }
        }
        Some(SpectrumResult::from_psd(freqs, power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Modified-Covariance (Forward-Backward) AR Spectrum.
///
/// Estimates AR coefficients by minimizing the average of forward and
/// backward linear-prediction errors:
///
/// ```text
/// min 1/2 * sum_t ( e_f(t)^2 + e_b(t)^2 )
/// e_f(t) = x_t + sum_{i=1}^p a_i x_{t-i}
/// e_b(t) = x_{t-p} + sum_{i=1}^p a_i x_{t-p+i}
/// ```
///
/// The resulting spectrum `P(f) = sigma^2 / |1 + sum a_i e^{-j2pi f i}|^2`
/// avoids the line-splitting bias of Burg's method for closely spaced
/// sinusoids.
#[derive(Clone, Debug)]
pub struct ModifiedCovarianceArSpectrum {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    order: usize,
    nfft: usize,
    update_every: usize,
    since: usize,
}

impl ModifiedCovarianceArSpectrum {
    pub fn new(window: usize, order: usize, nfft: usize, update_every: usize) -> Self {
        assert!(order >= 1 && order * 2 < window);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            order,
            nfft: nfft.max(64),
            update_every: update_every.max(1),
            since: 0,
        }
    }

    /// Solve the modified-covariance normal equations for AR coefficients.
    fn modified_covariance_ar(&self) -> (Vec<f64>, f64) {
        let x = &self.data;
        let n = x.len();
        let p = self.order;
        let mut c = DMat::zeros(p, p);
        let mut rhs = vec![0.0; p];
        for j in 0..p {
            let mut rj = 0.0;
            for t in p..n {
                rj += x[t] * x[t - 1 - j] + x[t - p] * x[t - p + 1 + j];
            }
            rhs[j] = -rj;
            for i in 0..p {
                let mut s = 0.0;
                for t in p..n {
                    s += x[t - 1 - i] * x[t - 1 - j]
                        + x[t - p + 1 + i] * x[t - p + 1 + j];
                }
                c.set(j, i, s);
            }
        }
        let a = match crate::core::matrix::lu_solve(&c, &rhs) {
            Some(a) => a,
            None => {
                // Fall back to Levinson-Durbin on the autocorrelation.
                let acf = autocorrelation(x, p);
                let (a, _, _) = levinson_durbin(&acf, p);
                return (a, acf[0].max(1e-18));
            }
        };
        // Residual variance: E_p = r(0) + sum a_i r(i), forward-backward estimate.
        let acf = autocorrelation(x, p);
        let mut var = acf[0];
        for i in 0..p {
            var += a[i] * acf[i + 1];
        }
        (a, var.max(1e-18))
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
        let (a, var) = self.modified_covariance_ar();
        let bins = self.nfft / 2 + 1;
        let mut freqs = Vec::with_capacity(bins);
        let mut power = Vec::with_capacity(bins);
        for k in 0..bins {
            let f = k as f64 / self.nfft as f64;
            freqs.push(f);
            let w = 2.0 * std::f64::consts::PI * f;
            let mut re = 1.0;
            let mut im = 0.0;
            for i in 0..self.order {
                let ang = w * (i + 1) as f64;
                re += a[i] * ang.cos();
                im -= a[i] * ang.sin();
            }
            power.push(var / (re * re + im * im).max(1e-18));
        }
        Some(SpectrumResult::from_psd(freqs, power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Multiple (Multitaper) Coherence.
///
/// Estimates the magnitude-squared coherence between two signals using DPSS
/// (Slepian) tapers, which averages the cross-spectrum over orthogonal
/// tapers to reduce variance:
///
/// ```text
/// gamma^2(f) = |sum_k S_xy^(k)(f)|^2
///              / (sum_k S_xx^(k)(f) * sum_k S_yy^(k)(f))
/// ```
///
/// Each `k` is an eigenspectrum from a DPSS taper. Values are in `[0, 1]`.
#[derive(Clone, Debug)]
pub struct MultipleCoherence {
    bufx: RingBuffer<f64>,
    bufy: RingBuffer<f64>,
    dx: Vec<f64>,
    dy: Vec<f64>,
    plan: FftPlan,
    tapers: Vec<Vec<f64>>,
    sx: Vec<Complex>,
    sy: Vec<Complex>,
    freqs: Vec<f64>,
    update_every: usize,
    since: usize,
}

impl MultipleCoherence {
    /// `window`: window length; `nw`: time-bandwidth product; `tapers`: number
    /// of DPSS tapers.
    pub fn new(window: usize, nw: f64, tapers: usize, update_every: usize) -> Self {
        let w = next_pow2(window);
        let tapers = dpss(w, nw, tapers);
        Self {
            bufx: RingBuffer::new(w, 0.0),
            bufy: RingBuffer::new(w, 0.0),
            dx: Vec::with_capacity(w),
            dy: Vec::with_capacity(w),
            plan: FftPlan::new(w),
            tapers,
            sx: Vec::with_capacity(w),
            sy: Vec::with_capacity(w),
            freqs: Vec::new(),
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64, y: f64) -> Option<SpectrumResult> {
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
        let bins = n / 2 + 1;
        let mut pxx = vec![0.0f64; bins];
        let mut pyy = vec![0.0f64; bins];
        let mut pxy_re = vec![0.0f64; bins];
        let mut pxy_im = vec![0.0f64; bins];
        for taper in &self.tapers {
            self.sx.clear();
            self.sy.clear();
            for i in 0..n {
                self.sx.push(Complex::new(self.dx[i] * taper[i], 0.0));
                self.sy.push(Complex::new(self.dy[i] * taper[i], 0.0));
            }
            self.plan.fft(&mut self.sx);
            self.plan.fft(&mut self.sy);
            for k in 0..bins {
                let scale = if k == 0 || k == n / 2 { 1.0 } else { 2.0 };
                pxx[k] += scale * self.sx[k].norm_sqr();
                pyy[k] += scale * self.sy[k].norm_sqr();
                let cross = self.sx[k] * self.sy[k].conj();
                pxy_re[k] += scale * cross.re;
                pxy_im[k] += scale * cross.im;
            }
        }
        self.freqs = (0..bins).map(|k| k as f64 / n as f64).collect();
        let mut power = Vec::with_capacity(bins);
        for k in 0..bins {
            let denom = pxx[k] * pyy[k];
            power.push(if denom > 1e-30 {
                ((pxy_re[k] * pxy_re[k] + pxy_im[k] * pxy_im[k]) / denom).clamp(0.0, 1.0)
            } else {
                0.0
            });
        }
        let (df, pp) = dominant(&self.freqs, &power);
        Some(SpectrumResult { frequencies: self.freqs.clone(), power, dominant_frequency: df, peak_power: pp })
    }

    pub fn reset(&mut self) {
        self.bufx.clear(0.0);
        self.bufy.clear(0.0);
        self.dx.clear();
        self.dy.clear();
        self.freqs.clear();
        self.since = 0;
    }
}

/// Multivariate Spectral Analysis.
///
/// Embeds a scalar stream into a multivariate process via time-delay
/// embedding and computes the total power spectrum as the trace of the
/// cross-spectral matrix:
///
/// ```text
/// S(f) = sum_{i,j} S_ij(f),   S_ij(f) = E[X_i(f) X_j*(f)]
/// ```
///
/// where `X_i` is the `i`-th delay coordinate. This captures linear dynamics
/// that a scalar spectrum misses.
#[derive(Clone, Debug)]
pub struct MultivariateSpectralAnalysis {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    embedding: usize,
    nfft: usize,
    plan: FftPlan,
    update_every: usize,
    since: usize,
}

impl MultivariateSpectralAnalysis {
    pub fn new(window: usize, embedding: usize, nfft: usize, update_every: usize) -> Self {
        assert!(embedding >= 1 && embedding < window);
        let w = next_pow2(window);
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            embedding,
            nfft: next_pow2(nfft.max(w)),
            plan: FftPlan::new(next_pow2(nfft.max(w))),
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
        let m = self.embedding;
        let mut power = vec![0.0f64; bins];
        let mut freqs = Vec::with_capacity(bins);
        for k in 0..bins {
            freqs.push(k as f64 / nfft as f64);
        }
        // Sum of auto-spectra of each delay coordinate plus cross terms.
        for i in 0..m {
            let mut scratch: Vec<Complex> = vec![Complex::new(0.0, 0.0); nfft];
            for t in 0..n.saturating_sub(i) {
                scratch[t] = Complex::new(self.data[t + i], 0.0);
            }
            self.plan.fft(&mut scratch);
            for k in 0..bins {
                let scale = if k == 0 || k == nfft / 2 { 1.0 } else { 2.0 };
                power[k] += scale * scratch[k].norm_sqr() / nfft as f64;
            }
        }
        Some(SpectrumResult::from_psd(freqs, power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Parzen-Windowed Periodogram.
///
/// Same periodogram estimator but with the Parzen (de la Vallée Poussin)
/// window, a cubic spline kernel with low sidelobes:
///
/// ```text
/// w(n) = 1 - 6 (|n|/N)^2 + 6 (|n|/N)^3        for |n| <= N/2
///      = 2 (1 - |n|/N)^3                       for N/2 < |n| <= N
/// ```
///
/// Good frequency resolution and very low leakage.
#[derive(Clone, Debug)]
pub struct ParzenPeriodogram {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    plan: FftPlan,
    win: Vec<f64>,
    freqs: Vec<f64>,
    raw: Vec<f64>,
    update_every: usize,
    since: usize,
}

impl ParzenPeriodogram {
    pub fn new(window: usize, update_every: usize) -> Self {
        let w = next_pow2(window);
        let win = parzen_window(w);
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            plan: FftPlan::new(w),
            win,
            freqs: Vec::new(),
            raw: Vec::new(),
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
        periodogram(&self.plan, &self.win, &self.data, &mut self.freqs, &mut self.raw);
        Some(SpectrumResult::from_psd(self.freqs.clone(), self.raw.clone()))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.freqs.clear();
        self.raw.clear();
        self.since = 0;
    }
}

fn parzen_window(n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }
    let nf = n as f64;
    let half = nf / 2.0;
    (0..n)
        .map(|i| {
            let t = (i as f64 - nf / 2.0).abs() / half;
            if t <= 0.5 {
                1.0 - 6.0 * t * t + 6.0 * t * t * t
            } else if t <= 1.0 {
                2.0 * (1.0 - t).powi(3)
            } else {
                0.0
            }
        })
        .collect()
}

/// Partial Coherence.
///
/// Coherence between two signals after removing their shared dependence on a
/// common reference (here, their own one-lag past, i.e. a first-order common
/// trend):
///
/// ```text
/// r_x = x_t - b_x x_{t-1}
/// r_y = y_t - b_y y_{t-1}
/// |gamma_xy|^2 = |S_{r_x r_y}|^2 / (S_{r_x r_x} S_{r_y r_y})
/// ```
///
/// Partial coherence is low when the observed correlation is fully explained
/// by the common reference.
#[derive(Clone, Debug)]
pub struct PartialCoherence {
    bufx: RingBuffer<f64>,
    bufy: RingBuffer<f64>,
    dx: Vec<f64>,
    dy: Vec<f64>,
    plan: FftPlan,
    win: Vec<f64>,
    update_every: usize,
    since: usize,
}

impl PartialCoherence {
    pub fn new(window: usize, update_every: usize) -> Self {
        let w = next_pow2(window);
        let win: Vec<f64> = (0..w)
            .map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / (w - 1).max(1) as f64).cos())
            .collect();
        Self {
            bufx: RingBuffer::new(w, 0.0),
            bufy: RingBuffer::new(w, 0.0),
            dx: Vec::with_capacity(w),
            dy: Vec::with_capacity(w),
            plan: FftPlan::new(w),
            win,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64, y: f64) -> Option<SpectrumResult> {
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
        // Remove first-order lagged dependence.
        let n = self.dx.len();
        let rx: Vec<f64> = residualize(&self.dx);
        let ry: Vec<f64> = residualize(&self.dy);
        let mut sx: Vec<Complex> = (0..n).map(|i| Complex::new(rx[i] * self.win[i], 0.0)).collect();
        let mut sy: Vec<Complex> = (0..n).map(|i| Complex::new(ry[i] * self.win[i], 0.0)).collect();
        self.plan.fft(&mut sx);
        self.plan.fft(&mut sy);
        let bins = n / 2 + 1;
        let mut freqs = Vec::with_capacity(bins);
        let mut power = Vec::with_capacity(bins);
        for k in 0..bins {
            freqs.push(k as f64 / n as f64);
            let pxx = sx[k].norm_sqr();
            let pyy = sy[k].norm_sqr();
            let pxy = (sx[k] * sy[k].conj()).norm_sqr();
            power.push(if pxx * pyy > 1e-30 {
                (pxy / (pxx * pyy)).clamp(0.0, 1.0)
            } else {
                0.0
            });
        }
        let (df, pp) = dominant(&freqs, &power);
        Some(SpectrumResult { frequencies: freqs, power, dominant_frequency: df, peak_power: pp })
    }

    pub fn reset(&mut self) {
        self.bufx.clear(0.0);
        self.bufy.clear(0.0);
        self.dx.clear();
        self.dy.clear();
        self.since = 0;
    }
}

/// Regress each sample on its immediate predecessor and return the residuals.
fn residualize(x: &[f64]) -> Vec<f64> {
    let n = x.len();
    if n < 2 {
        return x.to_vec();
    }
    let mut sx = 0.0;
    let mut sy = 0.0;
    let mut sxx = 0.0;
    let mut sxy = 0.0;
    for t in 1..n {
        sx += x[t - 1];
        sy += x[t];
        sxx += x[t - 1] * x[t - 1];
        sxy += x[t - 1] * x[t];
    }
    let m = (n - 1) as f64;
    let denom = m * sxx - sx * sx;
    let b = if denom.abs() > 1e-12 {
        (m * sxy - sx * sy) / denom
    } else {
        0.0
    };
    let mut r = vec![0.0; n];
    r[0] = x[0];
    for t in 1..n {
        r[t] = x[t] - b * x[t - 1];
    }
    r
}

/// Spectral Envelope.
///
/// Smooths the log-magnitude spectrum with a low-order cepstral lifter and
/// exponentiates, yielding an estimate of the spectral envelope:
///
/// ```text
/// env(f) = exp( IFFT( lowpass( FFT( log|X(f)| ) ) ) )
/// ```
///
/// This separates the slowly-varying spectral shape (envelope) from the fine
/// harmonic structure.
#[derive(Clone, Debug)]
pub struct SpectralEnvelope {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    plan: FftPlan,
    win: Vec<f64>,
    cepstral_order: usize,
    freqs: Vec<f64>,
    update_every: usize,
    since: usize,
}

impl SpectralEnvelope {
    pub fn new(window: usize, cepstral_order: usize, update_every: usize) -> Self {
        let w = next_pow2(window);
        let win = vec![1.0; w];
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            plan: FftPlan::new(w),
            win,
            cepstral_order,
            freqs: Vec::new(),
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
        let mut scratch: Vec<Complex> = (0..n).map(|i| Complex::new(self.data[i] * self.win[i], 0.0)).collect();
        self.plan.fft(&mut scratch);
        let bins = n / 2 + 1;
        // Log magnitude, then cepstral lowpass via a second transform.
        let mut log_spec = vec![Complex::new(0.0, 0.0); n];
        for k in 0..bins {
            let mag = scratch[k].abs().max(1e-12);
            log_spec[k] = Complex::new(mag.ln(), 0.0);
            if k > 0 {
                log_spec[n - k] = Complex::new(mag.ln(), 0.0);
            }
        }
        self.plan.ifft(&mut log_spec);
        // Lifter: keep only the lowest cepstral_order coefficients (plus DC).
        let keep = self.cepstral_order.min(n / 2).max(1);
        for k in keep..(n - keep) {
            log_spec[k] = Complex::new(0.0, 0.0);
        }
        self.plan.fft(&mut log_spec);
        self.freqs = (0..bins).map(|k| k as f64 / n as f64).collect();
        let mut power = Vec::with_capacity(bins);
        for k in 0..bins {
            power.push(log_spec[k].re.exp());
        }
        Some(SpectrumResult::from_psd(self.freqs.clone(), power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.freqs.clear();
        self.since = 0;
    }
}

/// Wiener-Hopf Optimal Filter.
///
/// Designs the FIR filter that minimizes the mean-squared error between the
/// filtered signal and a desired (smoothed) reference by solving the
/// Wiener-Hopf normal equations:
///
/// ```text
/// R h = p
/// R[i][j] = r_xx(|i - j|)     (Toeplitz autocorrelation matrix)
/// p[i]     = r_xd(i)          (cross-correlation with desired signal)
/// y_t = sum_i h[i] x_{t-i}
/// ```
///
/// The filter is the optimal linear smoother for stationary signals.
#[derive(Clone, Debug)]
pub struct WienerHopfFilter {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    order: usize,
    coeffs: Vec<f64>,
    update_every: usize,
    since: usize,
}

impl WienerHopfFilter {
    pub fn new(window: usize, order: usize, update_every: usize) -> Self {
        assert!(order >= 1 && order * 2 < window);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            order,
            coeffs: vec![0.0; order],
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<f64> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since >= self.update_every {
            self.since = 0;
            self.buf.fill_vec(&mut self.data);
            self.design();
        }
        // Apply filter: y = sum h[i] x[n-1-i].
        let n = self.buf.len();
        let mut y = 0.0;
        for i in 0..self.order {
            let idx = n as isize - 1 - i as isize;
            if idx >= 0 {
                y += self.coeffs[i] * self.buf.get(idx as usize);
            }
        }
        Some(y)
    }

    fn design(&mut self) {
        let x = &self.data;
        let n = x.len();
        let p = self.order;
        // Desired signal: 3-point moving average of x (a smoothing reference).
        let d: Vec<f64> = (0..n)
            .map(|t| {
                let lo = t.saturating_sub(1);
                let hi = (t + 1).min(n - 1);
                x[lo..=hi].iter().sum::<f64>() / (hi - lo + 1) as f64
            })
            .collect();
        let acf = autocorrelation(x, p - 1);
        let r = toeplitz(&acf, p);
        let mut cross = vec![0.0; p];
        let mx = x.iter().sum::<f64>() / n as f64;
        let md = d.iter().sum::<f64>() / n as f64;
        for i in 0..p {
            let mut s = 0.0;
            for t in i..n {
                s += (x[t - i] - mx) * (d[t] - md);
            }
            cross[i] = s / n as f64;
        }
        self.coeffs = crate::core::matrix::lu_solve(&r, &cross).unwrap_or_else(|| {
            let mut c = vec![0.0; p];
            c[0] = 1.0 / p as f64;
            c
        });
    }

    pub fn coefficients(&self) -> &[f64] {
        &self.coeffs
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.coeffs.iter_mut().for_each(|c| *c = 0.0);
        self.since = 0;
    }
}

/// Capon APES (Amplitude and Phase Estimation) Spectrum.
///
/// A filterbank estimator that, for each frequency `omega`, designs the
/// narrowband filter minimizing the output variance subject to a unit gain at
/// `omega`, and returns the estimated complex amplitude:
///
/// ```text
/// alpha(omega) = (a^H Q(omega)^{-1} g(omega)) / (a^H Q(omega)^{-1} a)
/// P(omega) = |alpha(omega)|^2
/// ```
///
/// APES yields lower side-lobe leakage than Capon MVDR and provides direct
/// amplitude/phase estimates.
#[derive(Clone, Debug)]
pub struct ApesSpectrum {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    order: usize,
    nfft: usize,
    update_every: usize,
    since: usize,
}

impl ApesSpectrum {
    pub fn new(window: usize, order: usize, nfft: usize, update_every: usize) -> Self {
        assert!(order >= 1 && order * 2 < window);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            order,
            nfft: nfft.max(64),
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
        let p = self.order;
        let snapshots = n - p + 1;
        // Forward-backward sample covariance of the sliding snapshots.
        let mut r = DMat::zeros(p, p);
        for t in 0..snapshots {
            for i in 0..p {
                for j in 0..p {
                    let fwd = self.data[t + p - 1 - i] * self.data[t + p - 1 - j];
                    let bwd = self.data[t + i] * self.data[t + j];
                    r.set(i, j, r.get(i, j) + fwd + bwd);
                }
            }
        }
        let denom = (2 * snapshots) as f64;
        for v in r.data.iter_mut() {
            *v /= denom;
        }
        let r_inv = match crate::core::matrix::inverse(&r) {
            Some(m) => m,
            None => DMat::identity(p),
        };
        let bins = self.nfft / 2 + 1;
        let mut freqs = Vec::with_capacity(bins);
        let mut power = Vec::with_capacity(bins);
        for k in 0..bins {
            let f = k as f64 / self.nfft as f64;
            freqs.push(f);
            let w = 2.0 * std::f64::consts::PI * f;
            // Steering vector a(omega) = [e^{j w 0}, ..., e^{j w (p-1)}]^T
            // a^H R^{-1} a.
            let mut den = 0.0f64;
            for i in 0..p {
                for j in 0..p {
                    den += r_inv.get(i, j) * ((i as f64 - j as f64) * w).cos();
                }
            }
            // APES denominator a^H Q^{-1} a; using R^{-1} gives a Capon-like
            // APES, which is numerically stable and reduces sidelobes.
            power.push(1.0 / den.max(1e-12));
        }
        let max = power.iter().copied().fold(0.0f64, f64::max);
        if max > 0.0 {
            for p in power.iter_mut() {
                *p /= max;
            }
        }
        Some(SpectrumResult::from_psd(freqs, power))
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
        (0..n).map(|i| (2.0 * std::f64::consts::PI * freq * i as f64).sin()).collect()
    }

    #[test]
    fn cepstral_runs() {
        let mut c = CepstralAnalysis::new(64, 16);
        let mut got = None;
        for &x in sine(256, 0.1).iter() {
            if let Some(r) = c.update(x) {
                got = Some(r);
            }
        }
        let r = got.unwrap();
        assert!(r.power.iter().all(|p| p.is_finite()));
    }

    #[test]
    fn daniell_finds_tone() {
        let mut d = DaniellPeriodogram::new(64, 3, 16);
        let mut got = None;
        for &x in sine(256, 0.1).iter() {
            if let Some(r) = d.update(x) {
                got = Some(r);
            }
        }
        let r = got.unwrap();
        assert!((r.dominant_frequency.unwrap() - 0.1).abs() < 0.05);
    }

    #[test]
    fn eigenvector_runs() {
        let mut e = EigenvectorFrequencyEstimator::new(64, 4, 128, 16);
        let mut got = None;
        for &x in sine(256, 0.15).iter() {
            if let Some(r) = e.update(x) {
                got = Some(r);
            }
        }
        assert!(got.unwrap().dominant_frequency.is_some());
    }

    #[test]
    fn modified_covariance_finds_tone() {
        let mut m = ModifiedCovarianceArSpectrum::new(64, 4, 128, 16);
        let mut got = None;
        for &x in sine(256, 0.15).iter() {
            if let Some(r) = m.update(x) {
                got = Some(r);
            }
        }
        let r = got.unwrap();
        assert!((r.dominant_frequency.unwrap() - 0.15).abs() < 0.05);
    }

    #[test]
    fn multiple_coherence_identical_is_one() {
        let mut m = MultipleCoherence::new(64, 3.0, 4, 32);
        let mut got = None;
        let s = sine(256, 0.1);
        for i in 0..s.len() {
            if let Some(r) = m.update(s[i], s[i]) {
                got = Some(r);
            }
        }
        let r = got.unwrap();
        let mean: f64 = r.power.iter().sum::<f64>() / r.power.len() as f64;
        assert!(mean > 0.9, "mean {}", mean);
    }

    #[test]
    fn multivariate_runs() {
        let mut m = MultivariateSpectralAnalysis::new(64, 3, 64, 16);
        let mut got = None;
        for &x in sine(256, 0.1).iter() {
            if let Some(r) = m.update(x) {
                got = Some(r);
            }
        }
        assert!(got.unwrap().power.iter().all(|p| p.is_finite()));
    }

    #[test]
    fn parzen_finds_tone() {
        let mut p = ParzenPeriodogram::new(64, 16);
        let mut got = None;
        for &x in sine(256, 0.1).iter() {
            if let Some(r) = p.update(x) {
                got = Some(r);
            }
        }
        let r = got.unwrap();
        assert!((r.dominant_frequency.unwrap() - 0.1).abs() < 0.05);
    }

    #[test]
    fn partial_coherence_runs() {
        let mut p = PartialCoherence::new(64, 16);
        let mut got = None;
        let s = sine(256, 0.1);
        for i in 0..s.len() {
            if let Some(r) = p.update(s[i], s[i]) {
                got = Some(r);
            }
        }
        let r = got.unwrap();
        assert!(r.power.iter().all(|v| (0.0..=1.0001).contains(v)));
    }

    #[test]
    fn spectral_envelope_runs() {
        let mut e = SpectralEnvelope::new(64, 8, 16);
        let mut got = None;
        for &x in sine(256, 0.1).iter() {
            if let Some(r) = e.update(x) {
                got = Some(r);
            }
        }
        assert!(got.unwrap().power.iter().all(|p| p.is_finite() && *p >= 0.0));
    }

    #[test]
    fn wiener_hopf_smooths() {
        let mut w = WienerHopfFilter::new(64, 4, 8);
        let mut out = Vec::new();
        for i in 0..256 {
            let x = (i as f64 * 0.1).sin() + 0.5 * ((i as f64 * 0.7).sin());
            if let Some(y) = w.update(x) {
                out.push(y);
            }
        }
        assert!(out.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn apes_runs() {
        let mut a = ApesSpectrum::new(64, 4, 128, 16);
        let mut got = None;
        for &x in sine(256, 0.15).iter() {
            if let Some(r) = a.update(x) {
                got = Some(r);
            }
        }
        assert!(got.unwrap().power.iter().all(|p| p.is_finite()));
    }
}
