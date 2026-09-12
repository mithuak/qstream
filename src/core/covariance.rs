//! Rolling/online covariance primitives and a multivariate EWMA covariance
//! matrix (RiskMetrics style) for portfolio risk.

use super::ring::RingBuffer;

/// Rolling covariance between two streams over a fixed window. O(1) update.
#[derive(Clone, Debug)]
pub struct RollingCovariance {
    bufx: RingBuffer<f64>,
    bufy: RingBuffer<f64>,
    sx: f64,
    sy: f64,
    sxy: f64,
}

impl RollingCovariance {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self {
            bufx: RingBuffer::new(period, 0.0),
            bufy: RingBuffer::new(period, 0.0),
            sx: 0.0,
            sy: 0.0,
            sxy: 0.0,
        }
    }
    #[inline]
    pub fn update(&mut self, x: f64, y: f64) -> f64 {
        let ex = self.bufx.push(x);
        let ey = self.bufy.push(y);
        // Both buffers share capacity and are pushed together, so they evict
        // in lockstep; subtract the evicted product term.
        if let (Some(ex), Some(ey)) = (ex, ey) {
            self.sx -= ex;
            self.sy -= ey;
            self.sxy -= ex * ey;
        }
        self.sx += x;
        self.sy += y;
        self.sxy += x * y;
        self.covariance()
    }
    #[inline]
    pub fn covariance(&self) -> f64 {
        let n = self.count() as f64;
        if n < 2.0 {
            return 0.0;
        }
        (self.sxy - self.sx * self.sy / n) / (n - 1.0)
    }
    #[inline]
    pub fn count(&self) -> usize {
        self.bufx.len()
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.bufx.is_full()
    }
    pub fn reset(&mut self) {
        self.bufx.clear(0.0);
        self.bufy.clear(0.0);
        self.sx = 0.0;
        self.sy = 0.0;
        self.sxy = 0.0;
    }
}

/// EWMA multivariate covariance matrix (RiskMetrics). State is `N x N`.
/// Update is O(N^2) recursive: `C = (1-l) C + l * x x^T`.
#[derive(Clone, Debug)]
pub struct EwmaCovariance {
    n: usize,
    lambda: f64,
    /// Row-major `n x n` covariance.
    cov: Vec<f64>,
    initialized: bool,
    /// Reusable scratch for the outer product accumulation.
    x: Vec<f64>,
}

impl EwmaCovariance {
    pub fn new(n: usize, lambda: f64) -> Self {
        assert!(n > 0, "dimension must be > 0");
        assert!((0.0..1.0).contains(&lambda), "lambda must be in (0,1)");
        Self {
            n,
            lambda,
            cov: vec![0.0; n * n],
            initialized: false,
            x: vec![0.0; n],
        }
    }

    /// RiskMetrics daily default lambda = 0.94.
    pub fn risk_metrics_daily(n: usize) -> Self {
        Self::new(n, 0.94)
    }

    #[inline]
    pub fn dim(&self) -> usize {
        self.n
    }

    /// Update with a return vector of length `n`.
    pub fn update(&mut self, returns: &[f64]) {
        assert_eq!(returns.len(), self.n, "return vector length mismatch");
        self.x.copy_from_slice(returns);
        if !self.initialized {
            // Seed with the outer product of the first observation.
            for i in 0..self.n {
                for j in 0..self.n {
                    self.cov[i * self.n + j] = self.x[i] * self.x[j];
                }
            }
            self.initialized = true;
        } else {
            let l = self.lambda;
            let om = 1.0 - l;
            for i in 0..self.n {
                let xi = self.x[i];
                let base = i * self.n;
                for j in 0..self.n {
                    self.cov[base + j] = om * self.cov[base + j] + l * xi * self.x[j];
                }
            }
        }
    }

    #[inline]
    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.cov[i * self.n + j]
    }

    /// Full covariance as a flat row-major slice.
    #[inline]
    pub fn as_slice(&self) -> &[f64] {
        &self.cov
    }

    /// Portfolio variance for a weight vector `w`: `w^T C w`.
    pub fn portfolio_variance(&self, w: &[f64]) -> f64 {
        assert_eq!(w.len(), self.n);
        let mut acc = 0.0;
        for i in 0..self.n {
            let wi = w[i];
            if wi == 0.0 {
                continue;
            }
            let base = i * self.n;
            for j in 0..self.n {
                acc += wi * w[j] * self.cov[base + j];
            }
        }
        acc
    }

    pub fn is_ready(&self) -> bool {
        self.initialized
    }

    pub fn reset(&mut self) {
        for c in self.cov.iter_mut() {
            *c = 0.0;
        }
        self.initialized = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ewma_cov_seeds_and_updates() {
        let mut c = EwmaCovariance::new(2, 0.5);
        c.update(&[1.0, 0.0]);
        assert!((c.get(0, 0) - 1.0).abs() < 1e-12);
        c.update(&[0.0, 1.0]);
        // C = 0.5*[[1,0],[0,0]] + 0.5*[[0,0],[0,1]]
        assert!((c.get(0, 0) - 0.5).abs() < 1e-12);
        assert!((c.get(1, 1) - 0.5).abs() < 1e-12);
    }
}
