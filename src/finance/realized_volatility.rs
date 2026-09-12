//! Realized volatility estimators with microstructure-noise correction
//! (design Phase 7, Tier C).
//!
//! Realized Kernel with microstructure-noise correction and Two-Scale
//! Realized Variance (TSRV). These provide noise-robust estimates of
//! integrated variance from high-frequency returns.

use crate::core::ring::RingBuffer;

/// Realized Kernel with Microstructure Noise Correction.
///
/// Computes a kernel-weighted realized variance that is robust to
/// microstructure noise. Uses a Parzen kernel to down-weight higher
/// autocovariances that are contaminated by noise.
///
/// The realized kernel is: RK = gamma_0 + sum_{h=1}^{H} k(h/(H+1)) * (gamma_h + gamma_{-h})
/// where gamma_h is the h-th order autocovariance of returns.
#[derive(Clone, Debug)]
pub struct RealizedKernel {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    n_lags: usize,
    annualization: f64,
    update_every: usize,
    since: usize,
}

impl RealizedKernel {
    /// `window`: rolling window length.
    /// `n_lags`: number of autocovariance lags (H).
    /// `annualization`: annualization factor (e.g. 252).
    pub fn new(window: usize, n_lags: usize, annualization: f64, update_every: usize) -> Self {
        assert!(window > 0 && n_lags > 0);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            n_lags: n_lags.min(window / 2),
            annualization,
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
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();
        let h = self.n_lags;

        // Compute mean return
        let mean_r: f64 = self.data.iter().sum::<f64>() / n as f64;

        // Compute autocovariances gamma_h for h = 0, 1, ..., H
        let mut gamma = vec![0.0f64; h + 1];
        for lag in 0..=h {
            let mut sum = 0.0f64;
            for i in lag..n {
                sum += (self.data[i] - mean_r) * (self.data[i - lag] - mean_r);
            }
            gamma[lag] = sum / n as f64;
        }

        // Parzen kernel weights
        let kernel = |x: f64| -> f64 {
            let ax = x.abs();
            if ax <= 0.5 {
                1.0 - 6.0 * ax * ax + 6.0 * ax * ax * ax
            } else if ax <= 1.0 {
                2.0 * (1.0 - ax).powi(3)
            } else {
                0.0
            }
        };

        // Realized kernel
        let mut rk = gamma[0];
        for lag in 1..=h {
            let w = kernel(lag as f64 / (h as f64 + 1.0));
            rk += w * (gamma[lag] + gamma[lag]); // gamma_{-h} = gamma_h for real data
        }

        Some(rk.max(0.0) * self.annualization)
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Two-Scale Realized Variance (TSRV).
///
/// Zhang, Mykland, Ait-Sahalia (2005) estimator that corrects for
/// microstructure noise by comparing realized variance at two different
/// sampling frequencies. TSRV = RV_{avg} - (n/K) * RV_{sparse}
/// where RV_{avg} is the average of K subsampled RV estimates.
#[derive(Clone, Debug)]
pub struct TwoScaleRealizedVariance {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    n_subsamples: usize,
    annualization: f64,
    update_every: usize,
    since: usize,
}

impl TwoScaleRealizedVariance {
    /// `window`: rolling window length.
    /// `n_subsamples`: number of subsamples K (e.g. 5-20).
    /// `annualization`: annualization factor.
    pub fn new(window: usize, n_subsamples: usize, annualization: f64, update_every: usize) -> Self {
        assert!(window > 0 && n_subsamples > 0);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            n_subsamples: n_subsamples.min(window),
            annualization,
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
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();
        let k = self.n_subsamples;

        // Sparse RV: sample every K observations
        let mut rv_sparse = 0.0f64;
        let mut prev = self.data[0];
        let mut count = 0;
        for i in (k..n).step_by(k) {
            let diff = self.data[i] - prev;
            rv_sparse += diff * diff;
            prev = self.data[i];
            count += 1;
        }
        if count > 0 {
            rv_sparse /= count as f64;
            rv_sparse *= n as f64; // Scale to full length
        }

        // Average RV: average of K subsampled RVs
        let mut rv_avg = 0.0f64;
        for sub in 0..k {
            let mut rv_sub = 0.0f64;
            let mut prev_sub = self.data[sub];
            for i in (sub + k..n).step_by(k) {
                let diff = self.data[i] - prev_sub;
                rv_sub += diff * diff;
                prev_sub = self.data[i];
            }
            rv_avg += rv_sub;
        }
        rv_avg /= k as f64;

        // TSRV = RV_avg - (n/K) * RV_sparse (scaled)
        let n_f = n as f64;
        let k_f = k as f64;
        let tsrv = rv_avg - (n_f / k_f) * rv_sparse / n_f;

        Some(tsrv.max(0.0) * self.annualization)
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

    #[test]
    fn realized_kernel_runs() {
        let mut rk = RealizedKernel::new(64, 10, 252.0, 16);
        for i in 0..256 {
            let r = (i as f64 * 0.05).sin() * 0.01;
            if let Some(v) = rk.update(r) {
                assert!(v >= 0.0 && v.is_finite());
            }
        }
    }

    #[test]
    fn tsrv_runs() {
        let mut tsrv = TwoScaleRealizedVariance::new(64, 5, 252.0, 16);
        for i in 0..256 {
            let r = (i as f64 * 0.05).sin() * 0.01;
            if let Some(v) = tsrv.update(r) {
                assert!(v >= 0.0 && v.is_finite());
            }
        }
    }
}
