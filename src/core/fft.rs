//! Radix-2 FFT with cached, reusable plans (no per-call plan allocation) and a
//! plan cache keyed by transform size. Power-of-two sizes only; spectral
//! estimators zero-pad/segment to a power of two.

use super::num::Complex;
use std::collections::HashMap;

#[inline]
pub fn is_pow2(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

#[inline]
pub fn next_pow2(n: usize) -> usize {
    let mut p = 1usize;
    while p < n {
        p <<= 1;
    }
    p
}

/// A reusable FFT plan for a fixed power-of-two size. Twiddle factors are
/// precomputed once; `fft`/`ifft` allocate nothing.
#[derive(Clone, Debug)]
pub struct FftPlan {
    n: usize,
    /// tw[k] = exp(-2*pi*i*k/n), k in 0..n/2
    tw: Vec<Complex>,
}

impl FftPlan {
    pub fn new(n: usize) -> Self {
        assert!(is_pow2(n), "FFT size must be a power of two, got {n}");
        let half = n / 2;
        let mut tw = Vec::with_capacity(half.max(1));
        for k in 0..half.max(1) {
            let ang = -2.0 * std::f64::consts::PI * (k as f64) / (n as f64);
            tw.push(Complex::new(ang.cos(), ang.sin()));
        }
        Self { n, tw }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.n
    }

    /// In-place forward FFT. `data.len()` must equal `self.n`.
    pub fn fft(&self, data: &mut [Complex]) {
        let n = self.n;
        debug_assert_eq!(data.len(), n);
        // Bit-reversal permutation.
        let mut j = 0usize;
        for i in 1..n {
            let mut bit = n >> 1;
            while j & bit != 0 {
                j ^= bit;
                bit >>= 1;
            }
            j ^= bit;
            if i < j {
                data.swap(i, j);
            }
        }
        // Butterflies.
        let mut len = 2usize;
        while len <= n {
            let half = len >> 1;
            let step = n / len;
            let mut i = 0usize;
            while i < n {
                for k in 0..half {
                    let w = self.tw[k * step];
                    let u = data[i + k];
                    let v = data[i + k + half] * w;
                    data[i + k] = u + v;
                    data[i + k + half] = u - v;
                }
                i += len;
            }
            len <<= 1;
        }
    }

    /// In-place inverse FFT (normalized by 1/n).
    pub fn ifft(&self, data: &mut [Complex]) {
        for d in data.iter_mut() {
            d.im = -d.im;
        }
        self.fft(data);
        let inv = 1.0 / self.n as f64;
        for d in data.iter_mut() {
            d.re *= inv;
            d.im = -d.im * inv;
        }
    }

    /// Compute the one-sided complex spectrum of a real input of length `n`.
    /// Writes `n/2 + 1` bins into `out` (cleared first). No allocation after
    /// `out` has been used once.
    pub fn rfft(&self, input: &[f64], scratch: &mut Vec<Complex>, out: &mut Vec<Complex>) {
        debug_assert_eq!(input.len(), self.n);
        scratch.clear();
        scratch.reserve(self.n);
        for i in 0..self.n {
            let v = if i < input.len() { input[i] } else { 0.0 };
            scratch.push(Complex::new(v, 0.0));
        }
        self.fft(scratch);
        let bins = self.n / 2 + 1;
        out.clear();
        out.reserve(bins);
        for k in 0..bins {
            out.push(scratch[k]);
        }
    }
}

/// Cache of FFT plans keyed by transform size, so plans are built at most once.
#[derive(Default)]
pub struct FftPlanCache {
    plans: HashMap<usize, FftPlan>,
}

impl FftPlanCache {
    pub fn new() -> Self {
        Self { plans: HashMap::new() }
    }
    /// Get a cloned plan for `n` (building and caching it on first use).
    pub fn get(&mut self, n: usize) -> FftPlan {
        if let Some(p) = self.plans.get(&n) {
            return p.clone();
        }
        let p = FftPlan::new(n);
        self.plans.insert(n, p.clone());
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fft_single_tone_peak() {
        let n = 16;
        let plan = FftPlan::new(n);
        let sig: Vec<f64> = (0..n).map(|i| (2.0 * std::f64::consts::PI * 2.0 * i as f64 / n as f64).cos()).collect();
        let mut scratch = Vec::new();
        let mut out = Vec::new();
        plan.rfft(&sig, &mut scratch, &mut out);
        // Peak magnitude should be at bin 2.
        let mut best = 0usize;
        let mut bestv = -1.0f64;
        for (k, c) in out.iter().enumerate() {
            let m = c.abs();
            if m > bestv {
                bestv = m;
                best = k;
            }
        }
        assert_eq!(best, 2);
    }

    #[test]
    fn fft_ifft_roundtrip() {
        let n = 8;
        let plan = FftPlan::new(n);
        let orig: Vec<Complex> = (0..n).map(|i| Complex::new(i as f64, 0.0)).collect();
        let mut data = orig.clone();
        plan.fft(&mut data);
        plan.ifft(&mut data);
        for i in 0..n {
            assert!((data[i].re - orig[i].re).abs() < 1e-9);
            assert!(data[i].im.abs() < 1e-9);
        }
    }
}
