//! Portfolio optimization strategies (design Phase 7, Tier C).
//!
//! Maximum Diversification and Exponentially Weighted Portfolio (EWP)
//! optimization. These operate on rolling covariance estimates.

use crate::core::matrix::{cholesky_solve, DMat};
use crate::core::ring::RingBuffer;

/// Maximum Diversification Portfolio.
///
/// Maximizes the diversification ratio: (w^T sigma) / sqrt(w^T Sigma w)
/// where sigma is the vector of asset volatilities. The optimal weights
/// are proportional to Sigma^{-1} sigma.
///
/// Streaming: updates on a rolling window using EWMA covariance.
#[derive(Clone, Debug)]
pub struct MaxDiversification {
    n_assets: usize,
    // Per-asset return buffers
    buffers: Vec<RingBuffer<f64>>,
    data: Vec<Vec<f64>>,
    period: usize,
    update_every: usize,
    since: usize,
}

impl MaxDiversification {
    pub fn new(n_assets: usize, period: usize, update_every: usize) -> Self {
        assert!(n_assets > 0 && n_assets <= 32);
        assert!(period > 0);
        Self {
            n_assets,
            buffers: (0..n_assets)
                .map(|_| RingBuffer::new(period, 0.0))
                .collect(),
            data: (0..n_assets).map(|_| Vec::with_capacity(period)).collect(),
            period,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    /// Update with a vector of asset returns.
    pub fn update(&mut self, returns: &[f64]) -> Option<Vec<f64>> {
        if returns.len() != self.n_assets {
            return None;
        }

        for (i, &r) in returns.iter().enumerate() {
            self.buffers[i].push(r);
        }

        // Check if all buffers are full
        if !self.buffers.iter().all(|b| b.is_full()) {
            return None;
        }

        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;

        // Fill data
        for (i, buf) in self.buffers.iter().enumerate() {
            self.data[i].clear();
            buf.fill_vec(&mut self.data[i]);
        }

        // Compute covariance matrix (sample covariance)
        let n = self.period as f64;
        let mut means = vec![0.0f64; self.n_assets];
        for i in 0..self.n_assets {
            means[i] = self.data[i].iter().sum::<f64>() / n;
        }

        let mut cov = DMat::zeros(self.n_assets, self.n_assets);
        for i in 0..self.n_assets {
            for j in 0..=i {
                let mut c = 0.0f64;
                for k in 0..self.period {
                    c += (self.data[i][k] - means[i]) * (self.data[j][k] - means[j]);
                }
                c /= (n - 1.0).max(1.0);
                cov.set(i, j, c);
                cov.set(j, i, c);
            }
        }

        // Compute volatilities (diagonal of covariance)
        let vols: Vec<f64> = (0..self.n_assets)
            .map(|i| cov.get(i, i).sqrt().max(1e-10))
            .collect();

        // Solve: w = Sigma^{-1} vols / (1^T Sigma^{-1} vols)
        let vols_vec = vols.clone();
        let w_unnorm = match cholesky_solve(&cov, &vols_vec) {
            Some(w) => w,
            None => {
                // Fallback: equal weights
                let eq = 1.0 / self.n_assets as f64;
                return Some(vec![eq; self.n_assets]);
            }
        };

        let sum_w: f64 = w_unnorm.iter().sum();
        if sum_w.abs() < 1e-30 {
            let eq = 1.0 / self.n_assets as f64;
            Some(vec![eq; self.n_assets])
        } else {
            Some(w_unnorm.iter().map(|w| w / sum_w).collect())
        }
    }

    pub fn reset(&mut self) {
        for buf in self.buffers.iter_mut() {
            buf.clear(0.0);
        }
        for d in self.data.iter_mut() {
            d.clear();
        }
        self.since = 0;
    }
}

/// Exponentially Weighted Portfolio (EWP).
///
/// Computes portfolio statistics using an exponentially weighted covariance
/// matrix. The decay factor `lambda` controls how much weight is given to
/// recent observations. Returns the minimum-variance portfolio weights.
#[derive(Clone, Debug)]
pub struct ExponentiallyWeightedPortfolio {
    n_assets: usize,
    lambda: f64,
    // EWMA state
    mean: Vec<f64>,
    cov: DMat,
    count: usize,
}

impl ExponentiallyWeightedPortfolio {
    pub fn new(n_assets: usize, lambda: f64) -> Self {
        assert!(n_assets > 0 && n_assets <= 32);
        assert!(lambda > 0.0 && lambda < 1.0);
        Self {
            n_assets,
            lambda,
            mean: vec![0.0; n_assets],
            cov: DMat::zeros(n_assets, n_assets),
            count: 0,
        }
    }

    /// Update with a vector of asset returns.
    pub fn update(&mut self, returns: &[f64]) -> Option<Vec<f64>> {
        if returns.len() != self.n_assets {
            return None;
        }

        let one_minus_lam = 1.0 - self.lambda;

        if self.count == 0 {
            // Initialize
            self.mean = returns.to_vec();
            self.count = 1;
            let eq = 1.0 / self.n_assets as f64;
            return Some(vec![eq; self.n_assets]);
        }

        // Update EWMA mean
        for i in 0..self.n_assets {
            self.mean[i] = self.lambda * self.mean[i] + one_minus_lam * returns[i];
        }

        // Update EWMA covariance
        for i in 0..self.n_assets {
            for j in 0..=i {
                let c = self.lambda * self.cov.get(i, j)
                    + one_minus_lam * (returns[i] - self.mean[i]) * (returns[j] - self.mean[j]);
                self.cov.set(i, j, c);
                self.cov.set(j, i, c);
            }
        }

        self.count += 1;

        // Compute minimum-variance portfolio: w = Sigma^{-1} 1 / (1^T Sigma^{-1} 1)
        let ones = vec![1.0f64; self.n_assets];
        let w_unnorm = match cholesky_solve(&self.cov, &ones) {
            Some(w) => w,
            None => {
                let eq = 1.0 / self.n_assets as f64;
                return Some(vec![eq; self.n_assets]);
            }
        };

        let sum_w: f64 = w_unnorm.iter().sum();
        if sum_w.abs() < 1e-30 {
            let eq = 1.0 / self.n_assets as f64;
            Some(vec![eq; self.n_assets])
        } else {
            Some(w_unnorm.iter().map(|w| w / sum_w).collect())
        }
    }

    /// Current portfolio volatility for given weights.
    pub fn portfolio_volatility(&self, weights: &[f64]) -> f64 {
        let c_w = self.cov.mul_vec(weights);
        let var: f64 = weights.iter().zip(c_w.iter()).map(|(w, c)| w * c).sum();
        var.max(0.0).sqrt()
    }

    pub fn reset(&mut self) {
        self.mean = vec![0.0; self.n_assets];
        self.cov = DMat::zeros(self.n_assets, self.n_assets);
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_diversification_runs() {
        let mut md = MaxDiversification::new(3, 32, 8);
        let mut weights = None;
        for i in 0..128 {
            let r = [
                (i as f64 * 0.1).sin() * 0.01,
                (i as f64 * 0.15).cos() * 0.01,
                (i as f64 * 0.05).sin() * 0.01,
            ];
            if let Some(w) = md.update(&r) {
                weights = Some(w);
            }
        }
        let w = weights.unwrap();
        assert_eq!(w.len(), 3);
        let sum: f64 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ewp_runs() {
        let mut ewp = ExponentiallyWeightedPortfolio::new(3, 0.94);
        let mut weights = None;
        for i in 0..128 {
            let r = [
                (i as f64 * 0.1).sin() * 0.01,
                (i as f64 * 0.15).cos() * 0.01,
                (i as f64 * 0.05).sin() * 0.01,
            ];
            if let Some(w) = ewp.update(&r) {
                weights = Some(w);
            }
        }
        let w = weights.unwrap();
        assert_eq!(w.len(), 3);
        let sum: f64 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);

        let vol = ewp.portfolio_volatility(&w);
        assert!(vol >= 0.0 && vol.is_finite());
    }
}
