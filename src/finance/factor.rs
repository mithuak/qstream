//! Factor models: CAPM beta/alpha, Jensen's alpha, rolling beta stability, and
//! N-factor online regressions (Fama-French 3/5, Carhart 4, generic
//! multi-factor, Treynor-Mazuy market timing) built on recursive least squares.

use crate::core::covariance::RollingCovariance;
use crate::core::moments::{RollingMean, RollingVariance};
use crate::core::regression::RecursiveLeastSquares;

/// Result of a factor-model update.
#[derive(Clone, Debug)]
pub struct FactorOutput {
    pub alpha: f64,
    pub betas: Vec<f64>,
    pub r2: f64,
    pub residual_variance: f64,
}

/// Rolling simple regression statistics for a pair `(x, y)` over a window,
/// exposing slope, intercept, correlation, R^2 and residual variance.
#[derive(Clone, Debug)]
pub struct RollingPairRegression {
    mx: RollingMean,
    my: RollingMean,
    vx: RollingVariance,
    vy: RollingVariance,
    cov: RollingCovariance,
}

impl RollingPairRegression {
    pub fn new(period: usize) -> Self {
        Self {
            mx: RollingMean::new(period),
            my: RollingMean::new(period),
            vx: RollingVariance::new(period),
            vy: RollingVariance::new(period),
            cov: RollingCovariance::new(period),
        }
    }
    pub fn update(&mut self, x: f64, y: f64) {
        self.mx.update(x);
        self.my.update(y);
        self.vx.update(x);
        self.vy.update(y);
        self.cov.update(x, y);
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.cov.is_ready()
    }
    #[inline]
    pub fn count(&self) -> usize {
        self.cov.count()
    }
    #[inline]
    pub fn slope(&self) -> f64 {
        let vx = self.vx.variance();
        if vx <= 1e-18 {
            0.0
        } else {
            self.cov.covariance() / vx
        }
    }
    #[inline]
    pub fn intercept(&self) -> f64 {
        self.my.value() - self.slope() * self.mx.value()
    }
    #[inline]
    pub fn correlation(&self) -> f64 {
        let vx = self.vx.variance();
        let vy = self.vy.variance();
        let d = (vx * vy).sqrt();
        if d <= 1e-18 {
            0.0
        } else {
            (self.cov.covariance() / d).clamp(-1.0, 1.0)
        }
    }
    #[inline]
    pub fn r2(&self) -> f64 {
        let c = self.correlation();
        c * c
    }
    #[inline]
    pub fn residual_variance(&self) -> f64 {
        self.vy.variance() * (1.0 - self.r2())
    }
    pub fn reset(&mut self) {
        self.mx.reset();
        self.my.reset();
        self.vx.reset();
        self.vy.reset();
        self.cov.reset();
    }
}

/// CAPM regression on excess returns: `(asset - rf) ~ alpha + beta (market - rf)`.
/// `alpha` is Jensen's alpha. Rolling window.
#[derive(Clone, Debug)]
pub struct CapmRegression {
    reg: RollingPairRegression,
    last_alpha: f64,
    last_beta: f64,
    last_r2: f64,
}

impl CapmRegression {
    pub fn new(period: usize) -> Self {
        Self { reg: RollingPairRegression::new(period), last_alpha: 0.0, last_beta: 0.0, last_r2: 0.0 }
    }
    /// Update with asset return, market return and per-period risk-free rate.
    /// Returns `(alpha, beta, r2)` once the window is warm.
    pub fn update(&mut self, asset: f64, market: f64, rf: f64) -> Option<(f64, f64, f64)> {
        self.reg.update(market - rf, asset - rf);
        if !self.reg.is_ready() {
            return None;
        }
        self.last_beta = self.reg.slope();
        self.last_alpha = self.reg.intercept();
        self.last_r2 = self.reg.r2();
        Some((self.last_alpha, self.last_beta, self.last_r2))
    }
    #[inline]
    pub fn alpha(&self) -> f64 {
        self.last_alpha
    }
    #[inline]
    pub fn beta(&self) -> f64 {
        self.last_beta
    }
    #[inline]
    pub fn r2(&self) -> f64 {
        self.last_r2
    }
    pub fn reset(&mut self) {
        self.reg.reset();
        self.last_alpha = 0.0;
        self.last_beta = 0.0;
        self.last_r2 = 0.0;
    }
}

/// Rolling beta of asset vs market (raw returns, no risk-free adjustment).
#[derive(Clone, Debug)]
pub struct RollingBeta {
    reg: RollingPairRegression,
}

impl RollingBeta {
    pub fn new(period: usize) -> Self {
        Self { reg: RollingPairRegression::new(period) }
    }
    pub fn update(&mut self, asset: f64, market: f64) -> Option<f64> {
        self.reg.update(market, asset);
        if self.reg.is_ready() {
            Some(self.reg.slope())
        } else {
            None
        }
    }
    #[inline]
    pub fn beta(&self) -> f64 {
        self.reg.slope()
    }
    #[inline]
    pub fn correlation(&self) -> f64 {
        self.reg.correlation()
    }
    pub fn reset(&mut self) {
        self.reg.reset();
    }
}

/// Rolling-window beta stability: standard deviation of the rolling beta over a
/// secondary window. Low values indicate a stable beta.
#[derive(Clone, Debug)]
pub struct RollingBetaStability {
    beta_window: usize,
    reg: RollingPairRegression,
    beta_var: RollingVariance,
    beta_count: usize,
}

impl RollingBetaStability {
    /// `beta_window` is the regression window; the beta dispersion uses the
    /// same window length.
    pub fn new(beta_window: usize, stability_window: usize) -> Self {
        Self {
            beta_window,
            reg: RollingPairRegression::new(beta_window),
            beta_var: RollingVariance::new(stability_window),
            beta_count: 0,
        }
    }
    pub fn update(&mut self, asset: f64, market: f64) -> Option<f64> {
        self.reg.update(market, asset);
        if !self.reg.is_ready() {
            return None;
        }
        let beta = self.reg.slope();
        self.beta_var.update(beta);
        self.beta_count += 1;
        if self.beta_count >= self.beta_window && self.beta_var.is_ready() {
            Some(self.beta_var.std_dev())
        } else if self.beta_var.is_ready() {
            Some(self.beta_var.std_dev())
        } else {
            None
        }
    }
    #[inline]
    pub fn beta(&self) -> f64 {
        self.reg.slope()
    }
    pub fn reset(&mut self) {
        self.reg.reset();
        self.beta_var.reset();
        self.beta_count = 0;
    }
}

/// Tracking error: rolling standard deviation of active returns
/// `(asset - benchmark)`.
#[derive(Clone, Debug)]
pub struct TrackingError {
    var: RollingVariance,
}

impl TrackingError {
    pub fn new(period: usize) -> Self {
        Self { var: RollingVariance::new(period) }
    }
    pub fn update(&mut self, asset: f64, benchmark: f64) -> Option<f64> {
        self.var.update(asset - benchmark);
        if self.var.is_ready() {
            Some(self.var.std_dev())
        } else {
            None
        }
    }
    pub fn reset(&mut self) {
        self.var.reset();
    }
}

/// Generic N-factor online regression via RLS. Excess asset return is regressed
/// on the factor vector. Provides alpha, betas, R^2 and residual variance.
#[derive(Clone, Debug)]
pub struct FactorModel {
    rls: RecursiveLeastSquares,
    n_factors: usize,
}

impl FactorModel {
    /// `lambda` is the RLS forgetting factor (1.0 = equal weighting).
    pub fn new(n_factors: usize, lambda: f64) -> Self {
        assert!(n_factors > 0, "need at least one factor");
        // Diffuse prior (large delta) so estimates converge regardless of the
        // small magnitude of typical factor returns (~1e-3).
        Self { rls: RecursiveLeastSquares::new(n_factors, lambda, 1e6), n_factors }
    }
    /// Fama-French 3-factor (market, SMB, HML).
    pub fn fama_french3(lambda: f64) -> Self {
        Self::new(3, lambda)
    }
    /// Fama-French 5-factor (market, SMB, HML, RMW, CMA).
    pub fn fama_french5(lambda: f64) -> Self {
        Self::new(5, lambda)
    }
    /// Carhart 4-factor (market, SMB, HML, MOM).
    pub fn carhart4(lambda: f64) -> Self {
        Self::new(4, lambda)
    }
    #[inline]
    pub fn n_factors(&self) -> usize {
        self.n_factors
    }
    /// Update with asset return, factor vector (length `n_factors`) and the
    /// per-period risk-free rate. Returns the current factor model output.
    pub fn update(&mut self, asset_return: f64, factors: &[f64], risk_free: f64) -> FactorOutput {
        assert_eq!(factors.len(), self.n_factors, "factor count mismatch");
        let excess = asset_return - risk_free;
        self.rls.update(factors, excess);
        FactorOutput {
            alpha: self.rls.intercept(),
            betas: self.rls.coefficients().to_vec(),
            r2: self.rls.r2(),
            residual_variance: self.rls.residual_variance(),
        }
    }
    /// Predict excess return for a factor vector.
    pub fn predict(&self, factors: &[f64]) -> f64 {
        self.rls.predict(factors)
    }
    #[inline]
    pub fn alpha(&self) -> f64 {
        self.rls.intercept()
    }
    #[inline]
    pub fn betas(&self) -> &[f64] {
        self.rls.coefficients()
    }
    #[inline]
    pub fn r2(&self) -> f64 {
        self.rls.r2()
    }
    #[inline]
    pub fn count(&self) -> usize {
        self.rls.count()
    }
    pub fn reset(&mut self) {
        self.rls.reset();
    }
}

/// Treynor-Mazuy market-timing model: regress excess asset return on
/// `[market_excess, market_excess^2]`. The quadratic coefficient `gamma`
/// measures market-timing ability.
#[derive(Clone, Debug)]
pub struct TreynorMazuy {
    rls: RecursiveLeastSquares,
}

impl TreynorMazuy {
    pub fn new(lambda: f64) -> Self {
        Self { rls: RecursiveLeastSquares::new(2, lambda, 1e6) }
    }
    pub fn update(&mut self, asset: f64, market: f64, rf: f64) -> FactorOutput {
        let m = market - rf;
        let excess = asset - rf;
        self.rls.update(&[m, m * m], excess);
        FactorOutput {
            alpha: self.rls.intercept(),
            betas: self.rls.coefficients().to_vec(),
            r2: self.rls.r2(),
            residual_variance: self.rls.residual_variance(),
        }
    }
    #[inline]
    pub fn alpha(&self) -> f64 {
        self.rls.intercept()
    }
    /// Market-timing (quadratic) coefficient.
    #[inline]
    pub fn gamma(&self) -> f64 {
        self.rls.coefficients().get(1).copied().unwrap_or(0.0)
    }
    pub fn reset(&mut self) {
        self.rls.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capm_recovers_beta_alpha() {
        let mut c = CapmRegression::new(50);
        // asset = 0.001 + 1.5*market exactly
        for i in 0..120 {
            let market = (i as f64) * 0.001 - 0.05;
            let asset = 0.001 + 1.5 * market;
            c.update(asset, market, 0.0);
        }
        assert!((c.beta() - 1.5).abs() < 1e-6);
        assert!((c.alpha() - 0.001).abs() < 1e-6);
        assert!(c.r2() > 0.999);
    }

    #[test]
    fn factor_model_three_factors() {
        let mut m = FactorModel::fama_french3(1.0);
        for i in 0..300 {
            let mkt = (i as f64 % 7.0) * 0.001 - 0.003;
            let smb = (i as f64 % 5.0) * 0.0005 - 0.001;
            let hml = (i as f64 % 3.0) * 0.0004 - 0.0004;
            let asset = 0.0002 + 1.1 * mkt + 0.5 * smb - 0.3 * hml;
            m.update(asset, &[mkt, smb, hml], 0.0);
        }
        let b = m.betas();
        assert!((b[0] - 1.1).abs() < 0.05);
        assert!((b[1] - 0.5).abs() < 0.05);
        assert!((b[2] + 0.3).abs() < 0.05);
        assert!(m.r2() > 0.95);
    }

    #[test]
    fn tracking_error_zero_when_identical() {
        let mut te = TrackingError::new(10);
        let mut v = None;
        for i in 0..20 {
            let r = (i as f64) * 0.001;
            v = te.update(r, r);
        }
        assert!(v.unwrap().abs() < 1e-12);
    }
}
