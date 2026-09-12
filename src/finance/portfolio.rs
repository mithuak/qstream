//! Portfolio analytics: return/duration aggregation, component & marginal VaR
//! from a streaming EWMA covariance, and equal-risk-contribution (risk parity)
//! weights.

use crate::core::covariance::EwmaCovariance;
use crate::finance::risk::probit;

/// Weighted portfolio return stream: `dot(returns, weights)`.
#[derive(Clone, Debug)]
pub struct PortfolioReturns {
    n: usize,
    cumulative: f64,
    count: u64,
}

impl PortfolioReturns {
    pub fn new(n: usize) -> Self {
        assert!(n > 0, "portfolio dimension must be > 0");
        Self { n, cumulative: 0.0, count: 0 }
    }
    /// Update with asset returns and weights (both length `n`). Returns the
    /// period portfolio return.
    pub fn update(&mut self, returns: &[f64], weights: &[f64]) -> f64 {
        assert_eq!(returns.len(), self.n);
        assert_eq!(weights.len(), self.n);
        let mut r = 0.0;
        for i in 0..self.n {
            r += returns[i] * weights[i];
        }
        self.cumulative += r;
        self.count += 1;
        r
    }
    #[inline]
    pub fn cumulative(&self) -> f64 {
        self.cumulative
    }
    #[inline]
    pub fn mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative / self.count as f64
        }
    }
    pub fn reset(&mut self) {
        self.cumulative = 0.0;
        self.count = 0;
    }
}

/// Aggregate portfolio duration: `dot(durations, weights)`.
#[derive(Clone, Debug)]
pub struct PortfolioDuration {
    n: usize,
    last: f64,
}

impl PortfolioDuration {
    pub fn new(n: usize) -> Self {
        assert!(n > 0, "portfolio dimension must be > 0");
        Self { n, last: 0.0 }
    }
    pub fn update(&mut self, durations: &[f64], weights: &[f64]) -> f64 {
        assert_eq!(durations.len(), self.n);
        assert_eq!(weights.len(), self.n);
        let mut d = 0.0;
        for i in 0..self.n {
            d += durations[i] * weights[i];
        }
        self.last = d;
        d
    }
    #[inline]
    pub fn value(&self) -> f64 {
        self.last
    }
    pub fn reset(&mut self) {
        self.last = 0.0;
    }
}

/// Component and marginal Value-at-Risk from a streaming EWMA covariance.
///
/// - portfolio volatility: `sqrt(w' C w)`
/// - marginal VaR_i: `z * (C w)_i / vol`
/// - component VaR_i: `w_i * marginal_i` (sums to total VaR `z * vol`)
#[derive(Clone, Debug)]
pub struct ComponentMarginalVaR {
    cov: EwmaCovariance,
    confidence: f64,
    z: f64,
    n: usize,
    cw: Vec<f64>,
    marginal: Vec<f64>,
    component: Vec<f64>,
}

impl ComponentMarginalVaR {
    pub fn new(n: usize, lambda: f64, confidence: f64) -> Self {
        Self {
            cov: EwmaCovariance::new(n, lambda),
            confidence,
            z: probit(confidence),
            n,
            cw: vec![0.0; n],
            marginal: vec![0.0; n],
            component: vec![0.0; n],
        }
    }
    /// Update the covariance with a new return vector.
    pub fn update(&mut self, returns: &[f64]) {
        self.cov.update(returns);
    }
    /// Recompute marginal/component VaR for the given weights. Returns
    /// `(total_var, marginal, component)`; marginal/component are the internal
    /// buffers (valid until the next call).
    pub fn compute(&mut self, weights: &[f64]) -> Option<(f64, &[f64], &[f64])> {
        assert_eq!(weights.len(), self.n);
        if !self.cov.is_ready() {
            return None;
        }
        // cw = C * w
        for i in 0..self.n {
            let mut acc = 0.0;
            for j in 0..self.n {
                acc += self.cov.get(i, j) * weights[j];
            }
            self.cw[i] = acc;
        }
        let mut pvar = 0.0;
        for i in 0..self.n {
            pvar += weights[i] * self.cw[i];
        }
        let vol = pvar.max(0.0).sqrt();
        if vol <= 1e-18 {
            for i in 0..self.n {
                self.marginal[i] = 0.0;
                self.component[i] = 0.0;
            }
            return Some((0.0, &self.marginal, &self.component));
        }
        for i in 0..self.n {
            self.marginal[i] = self.z * self.cw[i] / vol;
            self.component[i] = weights[i] * self.marginal[i];
        }
        let total_var = self.z * vol;
        Some((total_var, &self.marginal, &self.component))
    }
    #[inline]
    pub fn confidence(&self) -> f64 {
        self.confidence
    }
    pub fn reset(&mut self) {
        self.cov.reset();
        for i in 0..self.n {
            self.cw[i] = 0.0;
            self.marginal[i] = 0.0;
            self.component[i] = 0.0;
        }
    }
}

/// Equal-risk-contribution (risk parity) portfolio weights computed from a
/// covariance matrix via fixed-point iteration. Periodic optimization (Tier C);
/// not intended for per-tick updates.
pub struct RiskParity;

impl RiskParity {
    /// Compute ERC weights for a covariance matrix given as a flat row-major
    /// slice of length `n*n`. Returns normalized weights (length `n`).
    pub fn solve(cov: &[f64], n: usize, iters: usize) -> Vec<f64> {
        assert_eq!(cov.len(), n * n);
        // Initialize with inverse-volatility weights.
        let mut w = vec![0.0; n];
        for i in 0..n {
            let v = cov[i * n + i].max(1e-18);
            w[i] = 1.0 / v.sqrt();
        }
        normalize(&mut w);
        let mut cw = vec![0.0; n];
        for _ in 0..iters {
            // cw = C w
            for i in 0..n {
                let mut acc = 0.0;
                for j in 0..n {
                    acc += cov[i * n + j] * w[j];
                }
                cw[i] = acc;
            }
            // Risk contribution RC_i = w_i * cw_i. Equalize by scaling
            // w_i <- w_i * (mean_RC / RC_i), then renormalize.
            let mut rc = vec![0.0; n];
            let mut sum_rc = 0.0;
            for i in 0..n {
                rc[i] = w[i] * cw[i];
                sum_rc += rc[i];
            }
            let target = sum_rc / n as f64;
            for i in 0..n {
                let denom = rc[i].abs().max(1e-18);
                w[i] *= (target / denom).clamp(0.1, 10.0);
                if w[i] < 0.0 {
                    w[i] = 1e-6;
                }
            }
            normalize(&mut w);
        }
        w
    }
}

fn normalize(w: &mut [f64]) {
    let s: f64 = w.iter().sum();
    if s.abs() > 1e-18 {
        for x in w.iter_mut() {
            *x /= s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portfolio_return_dot() {
        let mut p = PortfolioReturns::new(2);
        let r = p.update(&[0.01, -0.02], &[0.5, 0.5]);
        assert!((r - (-0.005)).abs() < 1e-12);
    }

    #[test]
    fn risk_parity_equal_contribution() {
        // Two assets, identity covariance -> equal weights.
        let cov = vec![1.0, 0.0, 0.0, 1.0];
        let w = RiskParity::solve(&cov, 2, 200);
        assert!((w[0] - 0.5).abs() < 1e-3);
        assert!((w[1] - 0.5).abs() < 1e-3);
    }

    #[test]
    fn component_var_sums_to_total() {
        let mut c = ComponentMarginalVaR::new(2, 0.5, 0.95);
        for _ in 0..5 {
            c.update(&[0.01, 0.0]);
            c.update(&[0.0, 0.01]);
        }
        let (total, _marg, comp) = c.compute(&[0.5, 0.5]).unwrap();
        let sum: f64 = comp.iter().sum();
        assert!((sum - total).abs() < 1e-9);
    }
}
