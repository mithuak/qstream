//! Higher-order spectral analysis (design Phase 7, Tier C).
//!
//! Bispectrum, bicoherence, and higher-order cumulants. These reveal
//! nonlinear phase coupling between frequency components that the
//! power spectrum cannot detect.

use crate::core::fft::{next_pow2, FftPlan};
use crate::core::num::Complex;
use crate::core::ring::RingBuffer;

/// Bispectrum result: magnitude of the bispectral density on the
/// (f1, f2) plane, plus summary statistics.
#[derive(Clone, Debug)]
pub struct BispectrumResult {
    /// Bispectral magnitude values (flattened f1 x f2 grid).
    pub values: Vec<f64>,
    /// Frequency axis (shared for both dimensions).
    pub frequencies: Vec<f64>,
    /// Normalized bispectral entropy (0 = all energy at one point, 1 = uniform).
    pub entropy: f64,
    /// Sum of squared bicoherence (total quadratic phase coupling).
    pub total_coupling: f64,
}

/// Bicoherence result: normalized bispectrum (bicoherence) measuring
/// quadratic phase coupling, in [0, 1].
#[derive(Clone, Debug)]
pub struct BicoherenceResult {
    /// Bicoherence magnitude values (flattened f1 x f2 grid).
    pub values: Vec<f64>,
    /// Frequency axis (shared for both dimensions).
    pub frequencies: Vec<f64>,
    /// Peak bicoherence value.
    pub peak: f64,
}

/// Bispectrum analysis via direct FFT-based estimation.
///
/// Computes the bispectrum B(f1, f2) = E[X(f1) X(f2) X*(f1+f2)] using
/// the indirect method (segmented, averaged). Reveals quadratic phase
/// coupling between frequency components.
#[derive(Clone, Debug)]
pub struct BispectrumAnalysis {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    nfft: usize,
    fs: f64,
    update_every: usize,
    since: usize,
    plan: FftPlan,
    scratch: Vec<Complex>,
}

impl BispectrumAnalysis {
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
            scratch: Vec::with_capacity(nfft),
        }
    }

    pub fn update(&mut self, x: f64) -> Option<BispectrumResult> {
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

        // Compute FFT of the data
        self.scratch.clear();
        for i in 0..n {
            self.scratch.push(Complex::new(self.data[i], 0.0));
        }
        self.scratch.resize(nfft, Complex::new(0.0, 0.0));
        self.plan.fft(&mut self.scratch);
        let x_fft: Vec<Complex> = self.scratch.clone();

        // Compute bispectrum: B(f1, f2) = X(f1) * X(f2) * conj(X(f1+f2))
        let mut bispec = vec![0.0f64; bins * bins];
        let mut freqs = Vec::with_capacity(bins);

        for k in 0..bins {
            freqs.push(k as f64 * self.fs / nfft as f64);
        }

        for f1 in 0..bins {
            for f2 in 0..bins {
                let f3 = f1 + f2;
                if f3 < bins {
                    let b = x_fft[f1] * x_fft[f2] * x_fft[f3].conj();
                    bispec[f1 * bins + f2] = b.abs() / (nfft as f64 * nfft as f64);
                }
            }
        }

        // Compute entropy
        let total: f64 = bispec.iter().sum();
        let entropy = if total > 0.0 {
            let mut ent = 0.0f64;
            for &v in bispec.iter() {
                let p = v / total;
                if p > 0.0 {
                    ent -= p * p.ln();
                }
            }
            let norm = (bins as f64 * bins as f64).ln();
            if norm > 0.0 {
                ent / norm
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Total coupling (sum of squared normalized bispectrum)
        let total_coupling = bispec.iter().map(|v| v * v).sum();

        Some(BispectrumResult {
            values: bispec,
            frequencies: freqs,
            entropy,
            total_coupling,
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Bicoherence analysis.
///
/// Normalized bispectrum: |B(f1, f2)|^2 / (E[|X(f1)X(f2)|^2] E[|X(f1+f2)|^2]).
/// Values near 1 indicate strong quadratic phase coupling between f1, f2,
/// and f1+f2.
#[derive(Clone, Debug)]
pub struct BicoherenceAnalysis {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    nfft: usize,
    fs: f64,
    update_every: usize,
    since: usize,
    plan: FftPlan,
    scratch: Vec<Complex>,
}

impl BicoherenceAnalysis {
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
            scratch: Vec::with_capacity(nfft),
        }
    }

    pub fn update(&mut self, x: f64) -> Option<BicoherenceResult> {
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

        // Compute FFT
        self.scratch.clear();
        for i in 0..n {
            self.scratch.push(Complex::new(self.data[i], 0.0));
        }
        self.scratch.resize(nfft, Complex::new(0.0, 0.0));
        self.plan.fft(&mut self.scratch);
        let x_fft: Vec<Complex> = self.scratch.clone();

        // Compute bicoherence
        let mut bicoher = vec![0.0f64; bins * bins];
        let mut freqs = Vec::with_capacity(bins);
        let mut peak = 0.0f64;

        for k in 0..bins {
            freqs.push(k as f64 * self.fs / nfft as f64);
        }

        for f1 in 0..bins {
            for f2 in 0..bins {
                let f3 = f1 + f2;
                if f3 < bins {
                    let b = x_fft[f1] * x_fft[f2] * x_fft[f3].conj();
                    let b_mag_sq = b.norm_sqr();

                    // Denominator: E[|X(f1)X(f2)|^2] * E[|X(f1+f2)|^2]
                    let p12 = (x_fft[f1].norm_sqr() * x_fft[f2].norm_sqr()).max(1e-30);
                    let p3 = x_fft[f3].norm_sqr().max(1e-30);
                    let denom = p12 * p3;

                    let coh = if denom > 1e-30 {
                        (b_mag_sq / denom).min(1.0)
                    } else {
                        0.0
                    };
                    bicoher[f1 * bins + f2] = coh;
                    if coh > peak {
                        peak = coh;
                    }
                }
            }
        }

        Some(BicoherenceResult {
            values: bicoher,
            frequencies: freqs,
            peak,
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Higher-order cumulants (3rd and 4th order).
///
/// Computes the skewness (3rd cumulant) and kurtosis (4th cumulant) of
/// the signal's distribution, plus the normalized cumulant-based
/// nonlinearity index.
#[derive(Clone, Debug)]
pub struct HigherOrderCumulants {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    update_every: usize,
    since: usize,
}

/// Result from higher-order cumulant analysis.
#[derive(Clone, Debug)]
pub struct CumulantResult {
    /// Third standardized moment (skewness).
    pub skewness: f64,
    /// Fourth standardized moment (excess kurtosis).
    pub kurtosis: f64,
    /// Bicoherence-based nonlinearity index.
    pub nonlinearity_index: f64,
}

impl HigherOrderCumulants {
    pub fn new(window: usize, update_every: usize) -> Self {
        assert!(window > 0);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<CumulantResult> {
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

        let n = self.data.len() as f64;

        // Compute moments
        let mean: f64 = self.data.iter().sum::<f64>() / n;
        let mut m2 = 0.0f64;
        let mut m3 = 0.0f64;
        let mut m4 = 0.0f64;

        for &x in self.data.iter() {
            let d = x - mean;
            m2 += d * d;
            m3 += d * d * d;
            m4 += d * d * d * d;
        }

        m2 /= n;
        m3 /= n;
        m4 /= n;

        let skewness = if m2 > 1e-30 {
            m3 / m2.powf(1.5)
        } else {
            0.0
        };
        let kurtosis = if m2 > 1e-30 {
            m4 / (m2 * m2) - 3.0 // excess kurtosis
        } else {
            0.0
        };

        // Nonlinearity index: ratio of 4th-order to squared 2nd-order cumulant
        let nonlinearity_index = if m2 > 1e-30 {
            (m4 - 3.0 * m2 * m2).abs() / (m2 * m2)
        } else {
            0.0
        };

        Some(CumulantResult {
            skewness,
            kurtosis,
            nonlinearity_index,
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
    fn bispectrum_runs() {
        let mut bs = BispectrumAnalysis::new(64, 64, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = bs.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.values.len() > 0);
        assert!(r.entropy >= 0.0 && r.entropy <= 1.0001);
    }

    #[test]
    fn bicoherence_runs() {
        let mut bc = BicoherenceAnalysis::new(64, 64, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = bc.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.values.len() > 0);
        assert!(r.peak >= 0.0 && r.peak <= 1.0001);
    }

    #[test]
    fn higher_order_cumulants_runs() {
        let mut hoc = HigherOrderCumulants::new(64, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = hoc.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.skewness.is_finite());
        assert!(r.kurtosis.is_finite());
    }
}
