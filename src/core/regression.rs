//! Regression primitives: whole-history online simple regression, windowed
//! rolling simple regression, and matrix-based recursive least squares (RLS)
//! for multi-factor models and adaptive filtering.

use super::covariance::RollingCovariance;
use super::matrix::DMat;
use super::moments::{RollingMean, RollingVariance};

/// Whole-history simple linear regression (with intercept) via running sums.
/// O(1) update, no allocation. `y ~ a + b x`.
#[derive(Clone, Debug)]
pub struct OnlineSimpleRegression {
    n: f64,
    sx: f64,
    sy: f64,
    sxx: f64,
    sxy: f64,
    syy: f64,
}

impl OnlineSimpleRegression {
    pub fn new() -> Self {
        Self { n: 0.0, sx: 0.0, sy: 0.0, sxx: 0.0, sxy: 0.0, syy: 0.0 }
    }
    #[inline]
    pub fn update(&mut self, x: f64, y: f64) {
        self.n += 1.0;
        self.sx += x;
        self.sy += y;
        self.sxx += x * x;
        self.sxy += x * y;
        self.syy += y * y;
    }
    #[inline]
    pub fn count(&self) -> usize {
        self.n as usize
    }
    pub fn slope(&self) -> f64 {
        let denom = self.n * self.sxx - self.sx * self.sx;
        if denom.abs() < 1e-18 {
            0.0
        } else {
            (self.n * self.sxy - self.sx * self.sy) / denom
        }
    }
    pub fn intercept(&self) -> f64 {
        if self.n == 0.0 {
            0.0
        } else {
            (self.sy - self.slope() * self.sx) / self.n
        }
    }
    pub fn r2(&self) -> f64 {
        if self.n < 2.0 {
            return 0.0;
        }
        let sst = self.syy - self.sy * self.sy / self.n;
        if sst <= 1e-18 {
            return 0.0;
        }
        let sxy_c = self.sxy - self.sx * self.sy / self.n;
        let sxx_c = self.sxx - self.sx * self.sx / self.n;
        let ssr = sxy_c * sxy_c / sxx_c.max(1e-18);
        (1.0 - (sst - ssr) / sst).clamp(0.0, 1.0)
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for OnlineSimpleRegression {
    fn default() -> Self {
        Self::new()
    }
}

/// Rolling simple linear regression over a fixed window. `y ~ a + b x`.
/// Tier B: fixed-capacity buffers, O(1) update.
#[derive(Clone, Debug)]
pub struct RollingSimpleRegression {
    cov: RollingCovariance,
    varx: RollingVariance,
    meanx: RollingMean,
    meany: RollingMean,
}

impl RollingSimpleRegression {
    pub fn new(period: usize) -> Self {
        Self {
            cov: RollingCovariance::new(period),
            varx: RollingVariance::new(period),
            meanx: RollingMean::new(period),
            meany: RollingMean::new(period),
        }
    }
    pub fn update(&mut self, x: f64, y: f64) -> f64 {
        self.cov.update(x, y);
        self.varx.update(x);
        self.meanx.update(x);
        self.meany.update(y);
        self.slope()
    }
    #[inline]
    pub fn slope(&self) -> f64 {
        let vx = self.varx.variance();
        if vx <= 1e-18 {
            0.0
        } else {
            self.cov.covariance() / vx
        }
    }
    #[inline]
    pub fn intercept(&self) -> f64 {
        self.meany.value() - self.slope() * self.meanx.value()
    }
    #[inline]
    pub fn beta(&self) -> f64 {
        self.slope()
    }
    #[inline]
    pub fn count(&self) -> usize {
        self.cov.count()
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.cov.is_ready()
    }
    pub fn reset(&mut self) {
        self.cov.reset();
        self.varx.reset();
        self.meanx.reset();
        self.meany.reset();
    }
}

/// Multi-feature recursive least squares with intercept, forgetting factor,
/// and an online R^2 / residual-variance tracker. Used by factor models
/// (FF3/FF5/Carhart/multi-factor) and the RLS adaptive filter.
///
/// State: `theta` (p+1), covariance `P` ((p+1)x(p+1)). Update is O(p^2).
#[derive(Clone, Debug)]
pub struct RecursiveLeastSquares {
    p: usize,          // number of features (excluding intercept)
    dim: usize,        // p + 1
    lambda: f64,       // forgetting factor
    theta: Vec<f64>,   // coefficients [intercept, betas...]
    pd: DMat,          // inverse covariance
    u: Vec<f64>,       // regressor scratch (with intercept)
    pu: Vec<f64>,      // P*u scratch
    // online R^2 accumulators
    count: f64,
    sum_y: f64,
    sum_y2: f64,
    rss: f64,
}

impl RecursiveLeastSquares {
    pub fn new(p: usize, lambda: f64, delta: f64) -> Self {
        assert!(p > 0, "need at least one feature");
        assert!(lambda > 0.0 && lambda <= 1.0, "lambda in (0,1]");
        let dim = p + 1;
        let mut pd = DMat::identity(dim);
        pd.scale(delta);
        Self {
            p,
            dim,
            lambda,
            theta: vec![0.0; dim],
            pd,
            u: vec![0.0; dim],
            pu: vec![0.0; dim],
            count: 0.0,
            sum_y: 0.0,
            sum_y2: 0.0,
            rss: 0.0,
        }
    }

    #[inline]
    pub fn features(&self) -> usize {
        self.p
    }

    /// Update with feature vector `x` (length p) and target `y`.
    /// Returns the a-priori prediction error (y - prediction before update).
    pub fn update(&mut self, x: &[f64], y: f64) -> f64 {
        debug_assert_eq!(x.len(), self.p);
        // Build regressor u = [1, x...].
        self.u[0] = 1.0;
        self.u[1..].copy_from_slice(x);

        // pred = u^T theta
        let mut pred = 0.0;
        for i in 0..self.dim {
            pred += self.u[i] * self.theta[i];
        }
        let err = y - pred;

        // pu = P * u
        for i in 0..self.dim {
            let mut acc = 0.0;
            let base = i * self.dim;
            for j in 0..self.dim {
                acc += self.pd.data[base + j] * self.u[j];
            }
            self.pu[i] = acc;
        }
        // denom = lambda + u^T P u
        let mut utpu = 0.0;
        for i in 0..self.dim {
            utpu += self.u[i] * self.pu[i];
        }
        let denom = self.lambda + utpu;
        if denom.abs() < 1e-18 {
            return err;
        }
        // K = pu / denom ; theta += K * err
        for i in 0..self.dim {
            let k = self.pu[i] / denom;
            self.theta[i] += k * err;
        }
        // P = (P - K pu^T) / lambda  where K = pu/denom => K pu^T = pu pu^T / denom
        let inv_l = 1.0 / self.lambda;
        for i in 0..self.dim {
            let pui = self.pu[i];
            let base = i * self.dim;
            for j in 0..self.dim {
                self.pd.data[base + j] =
                    (self.pd.data[base + j] - pui * self.pu[j] / denom) * inv_l;
            }
        }

        // R^2 accumulators use the a-posteriori residual (after the update),
        // which excludes the cold-start transient that inflates a-priori error.
        let mut pred_post = 0.0;
        for i in 0..self.dim {
            pred_post += self.u[i] * self.theta[i];
        }
        let err_post = y - pred_post;
        self.count += 1.0;
        self.sum_y += y;
        self.sum_y2 += y * y;
        self.rss += err_post * err_post;
        err
    }

    /// Predict for a feature vector without updating state.
    pub fn predict(&self, x: &[f64]) -> f64 {
        debug_assert_eq!(x.len(), self.p);
        let mut acc = self.theta[0];
        for i in 0..self.p {
            acc += self.theta[i + 1] * x[i];
        }
        acc
    }

    #[inline]
    pub fn intercept(&self) -> f64 {
        self.theta[0]
    }
    #[inline]
    pub fn coefficients(&self) -> &[f64] {
        &self.theta[1..]
    }
    #[inline]
    pub fn theta(&self) -> &[f64] {
        &self.theta
    }
    #[inline]
    pub fn count(&self) -> usize {
        self.count as usize
    }

    /// Online R^2 = 1 - RSS/TSS.
    pub fn r2(&self) -> f64 {
        if self.count < 2.0 {
            return 0.0;
        }
        let mean = self.sum_y / self.count;
        let tss = self.sum_y2 - self.count * mean * mean;
        if tss <= 1e-18 {
            return 0.0;
        }
        (1.0 - self.rss / tss).clamp(-1.0, 1.0)
    }

    /// Residual variance (RSS / count).
    pub fn residual_variance(&self) -> f64 {
        if self.count < 1.0 {
            0.0
        } else {
            self.rss / self.count
        }
    }

    pub fn reset(&mut self) {
        let delta = self.pd.get(0, 0);
        self.theta.iter_mut().for_each(|t| *t = 0.0);
        self.pd = DMat::identity(self.dim);
        self.pd.scale(delta);
        self.count = 0.0;
        self.sum_y = 0.0;
        self.sum_y2 = 0.0;
        self.rss = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn online_simple_regression_exact_line() {
        let mut r = OnlineSimpleRegression::new();
        for i in 0..10 {
            let x = i as f64;
            let y = 2.0 + 3.0 * x;
            r.update(x, y);
        }
        assert!((r.slope() - 3.0).abs() < 1e-9);
        assert!((r.intercept() - 2.0).abs() < 1e-9);
        assert!((r.r2() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn rls_converges_to_line() {
        let mut r = RecursiveLeastSquares::new(1, 1.0, 100.0);
        for i in 0..200 {
            let x = (i as f64) * 0.01;
            let y = 1.5 + 2.0 * x;
            r.update(&[x], y);
        }
        assert!((r.coefficients()[0] - 2.0).abs() < 1e-3);
        assert!((r.intercept() - 1.5).abs() < 1e-2);
        assert!(r.r2() > 0.999);
    }
}
