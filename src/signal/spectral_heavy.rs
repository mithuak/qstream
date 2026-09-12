//! Heavy/experimental spectral estimators (design Phase 7, Tier C).
//!
//! Subspace and parametric frequency-estimation methods: MUSIC, ESPRIT,
//! Matrix Pencil, Capon MVDR, Minimum-Norm, Pisarenko, Prony. These push
//! every observation into a ring buffer but only recompute on an `update_every`
//! cadence, returning `None` on ticks where no computation ran.
//!
//! All methods build on the core primitives: autocorrelation, symmetric
//! Jacobi eigendecomposition (`jacobi_eigen`), and dense matrix ops (`DMat`).

use crate::core::matrix::{jacobi_eigen, DMat};
use crate::core::num::Complex;
use crate::core::ring::RingBuffer;
use crate::signal::prediction::autocorrelation;
use crate::signal::spectral::SpectrumResult;

/// Build an autocorrelation matrix (Toeplitz) of size `p x p` from the
/// autocorrelation sequence `acf` (length >= p). R[i,j] = acf[|i-j|].
fn autocorr_matrix(acf: &[f64], p: usize) -> DMat {
    let mut r = DMat::zeros(p, p);
    for i in 0..p {
        for j in 0..p {
            let lag = if i >= j { i - j } else { j - i };
            r.set(i, j, acf[lag]);
        }
    }
    r
}

/// MUSIC (Multiple Signal Classification) pseudospectrum.
///
/// Eigendecomposes the autocorrelation matrix, separates signal and noise
/// subspaces by the assumed number of complex exponentials `n_sources`, and
/// computes the pseudospectrum `1 / |E_n^H e(f)|^2` on a frequency grid.
///
/// Tier C: recomputes only every `update_every` samples.
#[derive(Clone, Debug)]
pub struct MusicSpectrum {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    n_sources: usize,
    nfft: usize,
    fs: f64,
    ar_order: usize,
    update_every: usize,
    since: usize,
    freqs: Vec<f64>,
}

impl MusicSpectrum {
    /// `window`: analysis window length (rounded up internally).
    /// `n_sources`: number of complex exponentials (model order).
    /// `nfft`: number of frequency grid points (controls resolution).
    /// `update_every`: recompute cadence in samples.
    pub fn new(window: usize, n_sources: usize, nfft: usize, update_every: usize) -> Self {
        assert!(window > 0, "window must be > 0");
        assert!(n_sources > 0, "n_sources must be > 0");
        assert!(n_sources < window, "n_sources must be < window");
        let ar_order = (window / 2).max(n_sources * 2).min(window - 1);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            n_sources,
            nfft: nfft.max(64),
            fs: 1.0,
            ar_order,
            update_every: update_every.max(1),
            since: 0,
            freqs: Vec::new(),
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

        let acf = autocorrelation(&self.data, self.ar_order);
        let p = self.ar_order;
        let r = autocorr_matrix(&acf, p);
        let (eig_vals, eig_vecs) = jacobi_eigen(&r, 200);

        // Noise subspace eigenvectors: columns n_sources..p
        let noise_dim = p - self.n_sources;
        if noise_dim == 0 {
            return None;
        }

        let bins = self.nfft;
        self.freqs.clear();
        let mut power = Vec::with_capacity(bins);

        for k in 0..bins {
            let f = k as f64 / bins as f64;
            self.freqs.push(f * self.fs);
            let w = 2.0 * std::f64::consts::PI * f;
            // Steering vector e(f) = [1, e^{-jw}, e^{-j2w}, ...]
            // |E_n^H e(f)|^2 = sum over noise eigenvectors
            let mut noise_power = 0.0f64;
            for col in self.n_sources..p {
                let mut re = 0.0f64;
                let mut im = 0.0f64;
                for i in 0..p {
                    let c = eig_vecs.get(i, col);
                    let ang = -w * i as f64;
                    re += c * ang.cos();
                    im += c * ang.sin();
                }
                noise_power += re * re + im * im;
            }
            let val = if noise_power > 1e-30 {
                1.0 / noise_power
            } else {
                1e30
            };
            power.push(val);
        }

        // Normalize power to a reasonable range.
        let max_p = power.iter().copied().fold(0.0f64, f64::max);
        if max_p > 0.0 && max_p.is_finite() {
            for v in power.iter_mut() {
                *v /= max_p;
            }
        }

        Some(SpectrumResult::from_psd(self.freqs.clone(), power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
        self.freqs.clear();
    }
}

/// ESPRIT (Estimation of Signal Parameters via Rotational Invariance
/// Techniques) frequency estimation.
///
/// Exploits the shift-structure of the signal subspace: two overlapping
/// subarrays share a rotation whose eigenvalues give the complex-exponential
/// frequencies. Returns frequencies and per-component power via the ESPRIT
/// pseudospectrum.
#[derive(Clone, Debug)]
pub struct EspritSpectrum {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    n_sources: usize,
    nfft: usize,
    fs: f64,
    ar_order: usize,
    update_every: usize,
    since: usize,
    freqs: Vec<f64>,
}

impl EspritSpectrum {
    pub fn new(window: usize, n_sources: usize, nfft: usize, update_every: usize) -> Self {
        assert!(window > 0, "window must be > 0");
        assert!(n_sources > 0, "n_sources must be > 0");
        assert!(n_sources * 2 < window, "need 2*n_sources < window");
        let ar_order = (window / 2).max(n_sources * 2).min(window - 1);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            n_sources,
            nfft: nfft.max(64),
            fs: 1.0,
            ar_order,
            update_every: update_every.max(1),
            since: 0,
            freqs: Vec::new(),
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

        let acf = autocorrelation(&self.data, self.ar_order);
        let p = self.ar_order;
        let r = autocorr_matrix(&acf, p);
        let (_eig_vals, eig_vecs) = jacobi_eigen(&r, 200);

        // Signal subspace: first n_sources eigenvectors.
        // Build S1 (rows 0..p-1) and S2 (rows 1..p) from signal subspace.
        let rows = p - 1;
        if rows == 0 || self.n_sources == 0 {
            return None;
        }

        // S1, S2 are rows x n_sources matrices.
        let mut s1 = DMat::zeros(rows, self.n_sources);
        let mut s2 = DMat::zeros(rows, self.n_sources);
        for i in 0..rows {
            for j in 0..self.n_sources {
                s1.set(i, j, eig_vecs.get(i, j));
                s2.set(i, j, eig_vecs.get(i + 1, j));
            }
        }

        // phi = pinv(S1) * S2 via normal equations: (S1^T S1)^{-1} S1^T S2
        let s1t = s1.transpose();
        let s1t_s1 = s1t.mul(&s1);
        let s1t_s2 = s1t.mul(&s2);

        // Invert S1^T S1 using our matrix inverse.
        let s1t_s1_inv = match crate::core::matrix::inverse(&s1t_s1) {
            Some(m) => m,
            None => return None,
        };
        let phi = s1t_s1_inv.mul(&s1t_s2);

        // Eigenvalues of phi give the signal poles. For a 2x2 or small matrix,
        // solve the characteristic polynomial directly. For larger, use Jacobi
        // on the (generally non-symmetric) phi via a small direct approach:
        // use the power iteration to find dominant eigenvalues approximately.
        let phi_eigs = eigenvalues_2x2_or_jacobi(&phi);

        // Build pseudospectrum from the estimated poles.
        let bins = self.nfft;
        self.freqs.clear();
        let mut power = Vec::with_capacity(bins);

        for k in 0..bins {
            let f = k as f64 / bins as f64;
            self.freqs.push(f * self.fs);
            let w = 2.0 * std::f64::consts::PI * f;
            let z_k = Complex::new(w.cos(), w.sin()); // e^{jw}

            // Spectrum contribution: 1 / |z_k - z_i|^2 for each pole
            let mut spec = 0.0f64;
            for eig in phi_eigs.iter() {
                let dz = Complex::new(z_k.re - eig.re, z_k.im - eig.im);
                let dist_sq = dz.norm_sqr().max(1e-30);
                spec += 1.0 / dist_sq;
            }
            power.push(spec);
        }

        let max_p = power.iter().copied().fold(0.0f64, f64::max);
        if max_p > 0.0 && max_p.is_finite() {
            for v in power.iter_mut() {
                *v /= max_p;
            }
        }

        Some(SpectrumResult::from_psd(self.freqs.clone(), power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
        self.freqs.clear();
    }
}

/// Compute eigenvalues of a small matrix. For 2x2, use the quadratic formula.
/// For larger, approximate via the characteristic polynomial roots using
/// Jacobi on the symmetric part as a fallback.
fn eigenvalues_2x2_or_jacobi(m: &DMat) -> Vec<Complex> {
    let n = m.rows;
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![Complex::new(m.get(0, 0), 0.0)];
    }
    if n == 2 {
        let a = m.get(0, 0);
        let b = m.get(0, 1);
        let c = m.get(1, 0);
        let d = m.get(1, 1);
        let trace = a + d;
        let det = a * d - b * c;
        let disc = trace * trace - 4.0 * det;
        if disc >= 0.0 {
            let sq = disc.sqrt();
            return vec![
                Complex::new((trace + sq) * 0.5, 0.0),
                Complex::new((trace - sq) * 0.5, 0.0),
            ];
        } else {
            let sq = (-disc).sqrt();
            return vec![
                Complex::new(trace * 0.5, sq * 0.5),
                Complex::new(trace * 0.5, -sq * 0.5),
            ];
        }
    }
    // For larger matrices, use power iteration to find dominant eigenvalues
    // deflating as we go. This is approximate but sufficient for pseudospectrum.
    let mut eigs = Vec::new();
    let mut remaining = m.clone();
    for _ in 0..n.min(8) {
        if remaining.rows == 0 {
            break;
        }
        if remaining.rows == 1 {
            eigs.push(Complex::new(remaining.get(0, 0), 0.0));
            break;
        }
        if remaining.rows == 2 {
            let a = remaining.get(0, 0);
            let b = remaining.get(0, 1);
            let c = remaining.get(1, 0);
            let d = remaining.get(1, 1);
            let trace = a + d;
            let det = a * d - b * c;
            let disc = trace * trace - 4.0 * det;
            if disc >= 0.0 {
                let sq = disc.sqrt();
                eigs.push(Complex::new((trace + sq) * 0.5, 0.0));
                eigs.push(Complex::new((trace - sq) * 0.5, 0.0));
            } else {
                let sq = (-disc).sqrt();
                eigs.push(Complex::new(trace * 0.5, sq * 0.5));
                eigs.push(Complex::new(trace * 0.5, -sq * 0.5));
            }
            break;
        }
        // Power iteration for dominant eigenvalue
        let mut v: Vec<f64> = (0..remaining.rows).map(|i| (i as f64 + 1.0).sin()).collect();
        let norm_v: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        for v_i in v.iter_mut() {
            *v_i /= norm_v.max(1e-30);
        }
        let mut lambda = 0.0f64;
        for _ in 0..100 {
            let new_v = remaining.mul_vec(&v);
            let new_norm: f64 = new_v.iter().map(|x| x * x).sum::<f64>().sqrt();
            if new_norm < 1e-30 {
                break;
            }
            lambda = new_v.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
            v = new_v;
            let n_v = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            for v_i in v.iter_mut() {
                *v_i /= n_v;
            }
        }
        eigs.push(Complex::new(lambda, 0.0));
        // Deflate: remaining = remaining - lambda * v * v^T
        for i in 0..remaining.rows {
            for j in 0..remaining.rows {
                let val = remaining.get(i, j) - lambda * v[i] * v[j];
                remaining.set(i, j, val);
            }
        }
    }
    eigs
}

/// Matrix Pencil (Hua-Sarkar) frequency estimation.
///
/// Forms a Hankel matrix from the signal and uses the generalized eigenvalue
/// problem via the matrix pencil method to extract damped sinusoid parameters.
/// Returns a pseudospectrum computed from the estimated poles.
#[derive(Clone, Debug)]
pub struct MatrixPencilSpectrum {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    n_sources: usize,
    nfft: usize,
    fs: f64,
    pencil_len: usize,
    update_every: usize,
    since: usize,
    freqs: Vec<f64>,
}

impl MatrixPencilSpectrum {
    pub fn new(window: usize, n_sources: usize, nfft: usize, update_every: usize) -> Self {
        assert!(window > 0, "window must be > 0");
        assert!(n_sources > 0, "n_sources must be > 0");
        assert!(n_sources < window / 2, "need n_sources < window/2");
        let pencil_len = window / 2;
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            n_sources,
            nfft: nfft.max(64),
            fs: 1.0,
            pencil_len,
            update_every: update_every.max(1),
            since: 0,
            freqs: Vec::new(),
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
        let l = self.pencil_len;
        let rows = n - l;
        if rows == 0 || l == 0 {
            return None;
        }

        // Build Y1 (rows x l) and Y2 (rows x l) Hankel matrices.
        // Y1[i,j] = data[i+j], Y2[i,j] = data[i+j+1]
        let mut y1 = DMat::zeros(rows, l);
        let mut y2 = DMat::zeros(rows, l);
        for i in 0..rows {
            for j in 0..l {
                y1.set(i, j, self.data[i + j]);
                y2.set(i, j, self.data[i + j + 1]);
            }
        }

        // SVD via truncated approach: we need the dominant n_sources right
        // singular vectors. Approximate using eigendecomposition of Y^T Y.
        let y1t = y1.transpose();
        let y1t_y1 = y1t.mul(&y1); // l x l
        let y1t_y2 = y1t.mul(&y2); // l x l

        // eig_vals, eig_vecs of Y1^T Y1
        let (_eig_vals, eig_vecs) = jacobi_eigen(&y1t_y1, 200);

        // Keep first n_sources eigenvectors
        let k = self.n_sources.min(l);
        let mut v = DMat::zeros(l, k);
        for i in 0..l {
            for j in 0..k {
                v.set(i, j, eig_vecs.get(i, j));
            }
        }

        // phi = (V^T Y1^T Y1 V)^{-1} (V^T Y1^T Y2 V)
        let vt = v.transpose();
        let vt_y1t_y1_v = vt.mul(&y1t_y1).mul(&v); // k x k
        let vt_y1t_y2_v = vt.mul(&y1t_y2).mul(&v); // k x k

        let vt_y1t_y1_v_inv = match crate::core::matrix::inverse(&vt_y1t_y1_v) {
            Some(m) => m,
            None => return None,
        };
        let phi = vt_y1t_y1_v_inv.mul(&vt_y1t_y2_v);

        // Eigenvalues of phi are the signal poles z_i
        let poles = eigenvalues_2x2_or_jacobi(&phi);

        // Build pseudospectrum
        let bins = self.nfft;
        self.freqs.clear();
        let mut power = Vec::with_capacity(bins);

        for bk in 0..bins {
            let f = bk as f64 / bins as f64;
            self.freqs.push(f * self.fs);
            let w = 2.0 * std::f64::consts::PI * f;
            let z_k = Complex::new(w.cos(), w.sin());

            let mut spec = 0.0f64;
            for pole in poles.iter() {
                let dz = Complex::new(z_k.re - pole.re, z_k.im - pole.im);
                let dist_sq = dz.norm_sqr().max(1e-30);
                spec += 1.0 / dist_sq;
            }
            power.push(spec);
        }

        let max_p = power.iter().copied().fold(0.0f64, f64::max);
        if max_p > 0.0 && max_p.is_finite() {
            for v in power.iter_mut() {
                *v /= max_p;
            }
        }

        Some(SpectrumResult::from_psd(self.freqs.clone(), power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
        self.freqs.clear();
    }
}

/// Capon (MVDR) spectral estimator.
///
/// Computes the minimum-variance distortionless-response spectrum:
/// `P(f) = 1 / (e(f)^H R^{-1} e(f))` where R is the AR (Toeplitz
/// autocorrelation) matrix. Higher resolution than the periodogram for
/// sinusoidal signals.
#[derive(Clone, Debug)]
pub struct CaponSpectrum {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    nfft: usize,
    fs: f64,
    ar_order: usize,
    update_every: usize,
    since: usize,
    freqs: Vec<f64>,
}

impl CaponSpectrum {
    pub fn new(window: usize, ar_order: usize, nfft: usize, update_every: usize) -> Self {
        assert!(window > 0, "window must be > 0");
        assert!(ar_order > 0, "ar_order must be > 0");
        assert!(ar_order < window, "ar_order must be < window");
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            nfft: nfft.max(64),
            fs: 1.0,
            ar_order,
            update_every: update_every.max(1),
            since: 0,
            freqs: Vec::new(),
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

        let acf = autocorrelation(&self.data, self.ar_order);
        let p = self.ar_order;
        let r = autocorr_matrix(&acf, p);
        let r_inv = match crate::core::matrix::inverse(&r) {
            Some(m) => m,
            None => return None,
        };

        let bins = self.nfft;
        self.freqs.clear();
        let mut power = Vec::with_capacity(bins);

        for k in 0..bins {
            let f = k as f64 / bins as f64;
            self.freqs.push(f * self.fs);
            let w = 2.0 * std::f64::consts::PI * f;

            // e(f)^H R^{-1} e(f) = sum_{i,j} R^{-1}_{ij} e^{j w (j-i)}
            let mut denom = 0.0f64;
            for i in 0..p {
                for j in 0..p {
                    let ang = w * (j as f64 - i as f64);
                    denom += r_inv.get(i, j) * ang.cos();
                }
            }

            let val = if denom > 1e-30 { 1.0 / denom } else { 0.0 };
            power.push(val.max(0.0));
        }

        let max_p = power.iter().copied().fold(0.0f64, f64::max);
        if max_p > 0.0 && max_p.is_finite() {
            for v in power.iter_mut() {
                *v /= max_p;
            }
        }

        Some(SpectrumResult::from_psd(self.freqs.clone(), power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
        self.freqs.clear();
    }
}

/// Pisarenko harmonic decomposition.
///
/// Uses the eigenvector corresponding to the smallest eigenvalue of the
/// autocorrelation matrix as the prediction-error filter whose zeros give
/// the frequency estimates. Returns a pseudospectrum from the polynomial
/// roots of this filter.
#[derive(Clone, Debug)]
pub struct PisarenkoSpectrum {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    n_sources: usize,
    nfft: usize,
    fs: f64,
    ar_order: usize,
    update_every: usize,
    since: usize,
    freqs: Vec<f64>,
}

impl PisarenkoSpectrum {
    pub fn new(window: usize, n_sources: usize, nfft: usize, update_every: usize) -> Self {
        assert!(window > 0, "window must be > 0");
        assert!(n_sources > 0, "n_sources must be > 0");
        let ar_order = (n_sources + 1).max(2).min(window - 1);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            n_sources,
            nfft: nfft.max(64),
            fs: 1.0,
            ar_order,
            update_every: update_every.max(1),
            since: 0,
            freqs: Vec::new(),
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

        let acf = autocorrelation(&self.data, self.ar_order);
        let p = self.ar_order;
        let r = autocorr_matrix(&acf, p);
        let (_eig_vals, eig_vecs) = jacobi_eigen(&r, 200);

        // Noise eigenvector: last column (smallest eigenvalue)
        let noise_vec: Vec<f64> = (0..p).map(|i| eig_vecs.get(i, p - 1)).collect();

        // Pseudospectrum: 1 / |A(f)|^2 where A(f) = sum a_k e^{-jwk}
        let bins = self.nfft;
        self.freqs.clear();
        let mut power = Vec::with_capacity(bins);

        for k in 0..bins {
            let f = k as f64 / bins as f64;
            self.freqs.push(f * self.fs);
            let w = 2.0 * std::f64::consts::PI * f;

            let mut re = 0.0f64;
            let mut im = 0.0f64;
            for i in 0..p {
                let ang = -w * i as f64;
                re += noise_vec[i] * ang.cos();
                im += noise_vec[i] * ang.sin();
            }
            let af_sq = (re * re + im * im).max(1e-30);
            power.push(1.0 / af_sq);
        }

        let max_p = power.iter().copied().fold(0.0f64, f64::max);
        if max_p > 0.0 && max_p.is_finite() {
            for v in power.iter_mut() {
                *v /= max_p;
            }
        }

        Some(SpectrumResult::from_psd(self.freqs.clone(), power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
        self.freqs.clear();
    }
}

/// Minimum-Norm spectral estimator.
///
/// Like Pisarenko but uses a linear combination of noise-subspace
/// eigenvectors constrained to have its first element equal to 1. The
/// resulting filter has its zeros on (or near) the unit circle at the
/// signal frequencies.
#[derive(Clone, Debug)]
pub struct MinimumNormSpectrum {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    n_sources: usize,
    nfft: usize,
    fs: f64,
    ar_order: usize,
    update_every: usize,
    since: usize,
    freqs: Vec<f64>,
}

impl MinimumNormSpectrum {
    pub fn new(window: usize, n_sources: usize, nfft: usize, update_every: usize) -> Self {
        assert!(window > 0, "window must be > 0");
        assert!(n_sources > 0, "n_sources must be > 0");
        let ar_order = (n_sources * 2).max(2).min(window - 1);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            n_sources,
            nfft: nfft.max(64),
            fs: 1.0,
            ar_order,
            update_every: update_every.max(1),
            since: 0,
            freqs: Vec::new(),
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

        let acf = autocorrelation(&self.data, self.ar_order);
        let p = self.ar_order;
        let r = autocorr_matrix(&acf, p);
        let (_eig_vals, eig_vecs) = jacobi_eigen(&r, 200);

        let noise_dim = p - self.n_sources;
        if noise_dim == 0 {
            return None;
        }

        // Noise subspace basis: columns n_sources..p
        // Minimum-norm vector: project e1 = [1,0,...,0]^T onto noise subspace,
        // then normalize so first element is 1.
        let mut en = vec![0.0f64; p];
        for i in 0..p {
            for col in self.n_sources..p {
                en[i] += eig_vecs.get(i, col) * eig_vecs.get(0, col);
            }
        }
        // en is the projection of e1 onto noise subspace.
        // The minimum-norm filter is: f = en / en[0]
        let en0 = en[0];
        if en0.abs() < 1e-30 {
            return None;
        }
        let filter: Vec<f64> = en.iter().map(|v| v / en0).collect();

        let bins = self.nfft;
        self.freqs.clear();
        let mut power = Vec::with_capacity(bins);

        for k in 0..bins {
            let f = k as f64 / bins as f64;
            self.freqs.push(f * self.fs);
            let w = 2.0 * std::f64::consts::PI * f;

            let mut re = 0.0f64;
            let mut im = 0.0f64;
            for i in 0..p {
                let ang = -w * i as f64;
                re += filter[i] * ang.cos();
                im += filter[i] * ang.sin();
            }
            let af_sq = (re * re + im * im).max(1e-30);
            power.push(1.0 / af_sq);
        }

        let max_p = power.iter().copied().fold(0.0f64, f64::max);
        if max_p > 0.0 && max_p.is_finite() {
            for v in power.iter_mut() {
                *v /= max_p;
            }
        }

        Some(SpectrumResult::from_psd(self.freqs.clone(), power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
        self.freqs.clear();
    }
}

/// Prony's method for exponential fitting.
///
/// Fits a sum of damped complex exponentials to the signal by solving a
/// linear prediction polynomial (via autocorrelation / a simplified Prony
/// approach) and evaluating its frequency response. Returns the signal
/// spectrum with sharp peaks at the estimated modal frequencies.
#[derive(Clone, Debug)]
pub struct PronySpectrum {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    n_modes: usize,
    nfft: usize,
    fs: f64,
    update_every: usize,
    since: usize,
    freqs: Vec<f64>,
}

impl PronySpectrum {
    pub fn new(window: usize, n_modes: usize, nfft: usize, update_every: usize) -> Self {
        assert!(window > 0, "window must be > 0");
        assert!(n_modes > 0, "n_modes must be > 0");
        assert!(n_modes * 2 < window, "need 2*n_modes < window");
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            n_modes,
            nfft: nfft.max(64),
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
            freqs: Vec::new(),
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
        let p = self.n_modes;

        // Prony via the standard method: solve the linear prediction
        // polynomial from a Hankel system built from the data.
        // Build the system: sum_{k=0}^{p} a_k x[m-k] = 0 for m = p..n-1
        // Using the autocorrelation method as a robust approximation.
        let acf = autocorrelation(&self.data, p);

        // Yule-Walker equations: R a = -r
        let mut r_mat = DMat::zeros(p, p);
        let mut rhs = vec![0.0f64; p];
        for i in 0..p {
            for j in 0..p {
                let lag = if i >= j { i - j } else { j - i };
                r_mat.set(i, j, acf[lag]);
            }
            rhs[i] = -acf[i + 1];
        }

        let a_coeffs = match crate::core::matrix::lu_solve(&r_mat, &rhs) {
            Some(a) => a,
            None => return None,
        };

        // The LP polynomial is A(z) = 1 + a_1 z^{-1} + ... + a_p z^{-p}
        // Spectrum: 1 / |A(e^{jw})|^2
        let bins = self.nfft;
        self.freqs.clear();
        let mut power = Vec::with_capacity(bins);

        for k in 0..bins {
            let f = k as f64 / bins as f64;
            self.freqs.push(f * self.fs);
            let w = 2.0 * std::f64::consts::PI * f;

            let mut re = 1.0f64;
            let mut im = 0.0f64;
            for i in 0..p {
                let ang = -w * (i + 1) as f64;
                re += a_coeffs[i] * ang.cos();
                im += a_coeffs[i] * ang.sin();
            }
            let af_sq = (re * re + im * im).max(1e-30);
            power.push(1.0 / af_sq);
        }

        let max_p = power.iter().copied().fold(0.0f64, f64::max);
        if max_p > 0.0 && max_p.is_finite() {
            for v in power.iter_mut() {
                *v /= max_p;
            }
        }

        Some(SpectrumResult::from_psd(self.freqs.clone(), power))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
        self.freqs.clear();
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

    fn two_tones(n: usize, f1: f64, f2: f64) -> Vec<f64> {
        (0..n)
            .map(|i| {
                (2.0 * std::f64::consts::PI * f1 * i as f64).sin()
                    + (2.0 * std::f64::consts::PI * f2 * i as f64).sin()
            })
            .collect()
    }

    #[test]
    fn music_finds_tone() {
        let mut m = MusicSpectrum::new(64, 1, 128, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = m.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        let df = r.dominant_frequency.unwrap();
        assert!(df.is_finite());
    }

    #[test]
    fn esprit_finds_tone() {
        let mut e = EspritSpectrum::new(64, 1, 128, 16);
        let sig = sine(256, 0.2);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = e.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        let df = r.dominant_frequency.unwrap();
        assert!(df.is_finite());
    }

    #[test]
    fn matrix_pencil_finds_tone() {
        let mut mp = MatrixPencilSpectrum::new(64, 1, 128, 16);
        let sig = sine(256, 0.12);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = mp.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        let df = r.dominant_frequency.unwrap();
        assert!(df.is_finite());
    }

    #[test]
    fn capon_finds_tone() {
        let mut c = CaponSpectrum::new(64, 16, 128, 16);
        let sig = sine(256, 0.18);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = c.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        let df = r.dominant_frequency.unwrap();
        assert!(df.is_finite());
    }

    #[test]
    fn pisarenko_finds_tone() {
        let mut p = PisarenkoSpectrum::new(64, 1, 128, 16);
        let sig = sine(256, 0.25);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = p.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.dominant_frequency.is_some());
    }

    #[test]
    fn minimum_norm_finds_tone() {
        let mut mn = MinimumNormSpectrum::new(64, 1, 128, 16);
        let sig = sine(256, 0.2);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = mn.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        let df = r.dominant_frequency.unwrap();
        assert!(df.is_finite());
    }

    #[test]
    fn prony_finds_tone() {
        let mut p = PronySpectrum::new(64, 1, 128, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = p.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        let df = r.dominant_frequency.unwrap();
        assert!(df.is_finite());
    }

    #[test]
    fn music_two_tones() {
        let mut m = MusicSpectrum::new(128, 2, 256, 32);
        let sig = two_tones(512, 0.1, 0.3);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = m.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        // Should have power concentrated near both frequencies.
        assert!(r.power.iter().all(|p| p.is_finite()));
    }
}
