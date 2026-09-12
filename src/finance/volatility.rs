//! Volatility estimators: range-based OHLC estimators (Parkinson, Garman-Klass,
//! Rogers-Satchell), recursive variance models (RiskMetrics EWMA, GARCH(1,1)),
//! realized volatility, and rolling volatility.
//!
//! Window-based estimators return `None` until warm-up completes.

use crate::core::moments::{Ewma, RollingMean, RollingSum, RollingVariance};
use crate::core::traits::{OhlcIndicator, PairIndicator, ScalarIndicator};

const LN2: f64 = std::f64::consts::LN_2;

/// Parkinson (1980) high-low range volatility over a rolling window.
/// `sigma^2 = mean(ln(H/L)^2) / (4 ln 2)`.
#[derive(Clone, Debug)]
pub struct Parkinson {
    period: usize,
    acc: RollingMean,
}

impl Parkinson {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self { period, acc: RollingMean::new(period) }
    }
    #[inline]
    pub fn period(&self) -> usize {
        self.period
    }
    #[inline]
    pub fn value(&self) -> Option<f64> {
        if !self.acc.is_ready() {
            return None;
        }
        Some((self.acc.value() / (4.0 * LN2)).max(0.0).sqrt())
    }
}

impl PairIndicator for Parkinson {
    type Output = f64;
    fn update(&mut self, high: f64, low: f64) -> Option<f64> {
        let r = (high / low).ln();
        self.acc.update(r * r);
        self.value()
    }
    fn reset(&mut self) {
        self.acc.reset();
    }
}

/// Garman-Klass (1980) OHLC volatility over a rolling window.
/// `sigma^2 = mean(0.5 ln(H/L)^2 - (2 ln2 - 1) ln(C/O)^2)`.
#[derive(Clone, Debug)]
pub struct GarmanKlass {
    period: usize,
    acc: RollingMean,
}

impl GarmanKlass {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self { period, acc: RollingMean::new(period) }
    }
    #[inline]
    pub fn period(&self) -> usize {
        self.period
    }
    #[inline]
    pub fn value(&self) -> Option<f64> {
        if !self.acc.is_ready() {
            return None;
        }
        Some(self.acc.value().max(0.0).sqrt())
    }
}

impl OhlcIndicator for GarmanKlass {
    type Output = f64;
    fn update(&mut self, open: f64, high: f64, low: f64, close: f64) -> Option<f64> {
        let hl = (high / low).ln();
        let co = (close / open).ln();
        let term = 0.5 * hl * hl - (2.0 * LN2 - 1.0) * co * co;
        self.acc.update(term);
        self.value()
    }
    fn reset(&mut self) {
        self.acc.reset();
    }
}

/// Rogers-Satchell (1991) drift-independent OHLC volatility.
/// `sigma^2 = mean(ln(H/C)ln(H/O) + ln(L/C)ln(L/O))`.
#[derive(Clone, Debug)]
pub struct RogersSatchell {
    period: usize,
    acc: RollingMean,
}

impl RogersSatchell {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self { period, acc: RollingMean::new(period) }
    }
    #[inline]
    pub fn period(&self) -> usize {
        self.period
    }
    #[inline]
    pub fn value(&self) -> Option<f64> {
        if !self.acc.is_ready() {
            return None;
        }
        Some(self.acc.value().max(0.0).sqrt())
    }
}

impl OhlcIndicator for RogersSatchell {
    type Output = f64;
    fn update(&mut self, open: f64, high: f64, low: f64, close: f64) -> Option<f64> {
        let term = (high / close).ln() * (high / open).ln()
            + (low / close).ln() * (low / open).ln();
        self.acc.update(term);
        self.value()
    }
    fn reset(&mut self) {
        self.acc.reset();
    }
}

/// RiskMetrics EWMA volatility forecast: `var_t = l*var_{t-1} + (1-l)*r_t^2`.
/// `lambda = 0.94` (daily) by default. O(1) recursive, no warm-up gate.
#[derive(Clone, Debug)]
pub struct EwmaVolatility {
    lambda: f64,
    variance: f64,
    initialized: bool,
}

impl EwmaVolatility {
    pub fn new(lambda: f64) -> Self {
        assert!((0.0..1.0).contains(&lambda), "lambda must be in (0,1)");
        Self { lambda, variance: 0.0, initialized: false }
    }
    /// RiskMetrics daily default.
    pub fn risk_metrics() -> Self {
        Self::new(0.94)
    }
    #[inline]
    pub fn variance(&self) -> f64 {
        self.variance
    }
    #[inline]
    pub fn value(&self) -> f64 {
        self.variance.max(0.0).sqrt()
    }
}

impl ScalarIndicator for EwmaVolatility {
    type Output = f64;
    fn update(&mut self, r: f64) -> Option<f64> {
        let r2 = r * r;
        if !self.initialized {
            self.variance = r2;
            self.initialized = true;
        } else {
            self.variance = self.lambda * self.variance + (1.0 - self.lambda) * r2;
        }
        Some(self.value())
    }
    fn reset(&mut self) {
        self.variance = 0.0;
        self.initialized = false;
    }
}

/// GARCH(1,1) conditional volatility: `s2_t = w + a*r_{t-1}^2 + b*s2_{t-1}`.
#[derive(Clone, Debug)]
pub struct Garch11 {
    omega: f64,
    alpha: f64,
    beta: f64,
    variance: f64,
    prev_r2: f64,
    initialized: bool,
}

impl Garch11 {
    pub fn new(omega: f64, alpha: f64, beta: f64) -> Self {
        assert!(omega >= 0.0, "omega must be >= 0");
        assert!(alpha >= 0.0 && beta >= 0.0, "alpha/beta must be >= 0");
        let long_run = if alpha + beta < 1.0 && alpha + beta > 0.0 {
            omega / (1.0 - alpha - beta)
        } else {
            omega.max(1e-12)
        };
        Self {
            omega,
            alpha,
            beta,
            variance: long_run,
            prev_r2: long_run,
            initialized: false,
        }
    }
    /// Common daily calibration.
    pub fn default_params() -> Self {
        Self::new(1e-6, 0.09, 0.90)
    }
    #[inline]
    pub fn variance(&self) -> f64 {
        self.variance
    }
    #[inline]
    pub fn value(&self) -> f64 {
        self.variance.max(0.0).sqrt()
    }
}

impl ScalarIndicator for Garch11 {
    type Output = f64;
    fn update(&mut self, r: f64) -> Option<f64> {
        let r2 = r * r;
        if !self.initialized {
            self.variance = self.omega + (self.alpha + self.beta) * r2;
            self.initialized = true;
        } else {
            self.variance = self.omega + self.alpha * self.prev_r2 + self.beta * self.variance;
        }
        self.prev_r2 = r2;
        Some(self.value())
    }
    fn reset(&mut self) {
        let long_run = if self.alpha + self.beta < 1.0 && self.alpha + self.beta > 0.0 {
            self.omega / (1.0 - self.alpha - self.beta)
        } else {
            self.omega.max(1e-12)
        };
        self.variance = long_run;
        self.prev_r2 = long_run;
        self.initialized = false;
    }
}

/// Realized volatility: `sqrt(sum(r^2))` over a rolling window.
#[derive(Clone, Debug)]
pub struct RealizedVolatility {
    period: usize,
    acc: RollingSum,
}

impl RealizedVolatility {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self { period, acc: RollingSum::new(period) }
    }
    #[inline]
    pub fn period(&self) -> usize {
        self.period
    }
    #[inline]
    pub fn variance(&self) -> Option<f64> {
        if self.acc.is_ready() {
            Some(self.acc.value())
        } else {
            None
        }
    }
}

impl ScalarIndicator for RealizedVolatility {
    type Output = f64;
    fn update(&mut self, r: f64) -> Option<f64> {
        self.acc.update(r * r);
        if !self.acc.is_ready() {
            return None;
        }
        Some(self.acc.value().max(0.0).sqrt())
    }
    fn reset(&mut self) {
        self.acc.reset();
    }
}

/// Rolling volatility: sample standard deviation of returns over a window.
#[derive(Clone, Debug)]
pub struct RollingVolatility {
    period: usize,
    acc: RollingVariance,
}

impl RollingVolatility {
    pub fn new(period: usize) -> Self {
        assert!(period > 1, "period must be > 1");
        Self { period, acc: RollingVariance::new(period) }
    }
    #[inline]
    pub fn period(&self) -> usize {
        self.period
    }
}

impl ScalarIndicator for RollingVolatility {
    type Output = f64;
    fn update(&mut self, r: f64) -> Option<f64> {
        self.acc.update(r);
        if !self.acc.is_ready() {
            return None;
        }
        Some(self.acc.std_dev())
    }
    fn reset(&mut self) {
        self.acc.reset();
    }
}

/// Rolling returns: windowed cumulative return `prod(1+r) - 1`.
#[derive(Clone, Debug)]
pub struct RollingReturns {
    buf: crate::core::ring::RingBuffer<f64>,
}

impl RollingReturns {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self { buf: crate::core::ring::RingBuffer::new(period, 0.0) }
    }
}

impl ScalarIndicator for RollingReturns {
    type Output = f64;
    fn update(&mut self, r: f64) -> Option<f64> {
        self.buf.push(1.0 + r);
        if !self.buf.is_full() {
            return None;
        }
        let mut prod = 1.0;
        for i in 0..self.buf.len() {
            prod *= *self.buf.get(i);
        }
        Some(prod - 1.0)
    }
    fn reset(&mut self) {
        self.buf.clear(0.0);
    }
}

/// Exponentially weighted variance used as a generic EWMA-of-squares building
/// block (exposed for parity with the workbook's "EWMA" family).
#[derive(Clone, Debug)]
pub struct EwmaVariance {
    inner: Ewma,
}

impl EwmaVariance {
    pub fn new(alpha: f64) -> Self {
        Self { inner: Ewma::new(alpha) }
    }
}

impl ScalarIndicator for EwmaVariance {
    type Output = f64;
    fn update(&mut self, r: f64) -> Option<f64> {
        Some(self.inner.update(r * r))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ewma_volatility_recursion() {
        let mut v = EwmaVolatility::new(0.5);
        let a = v.update(0.02).unwrap();
        assert!((a - 0.02).abs() < 1e-12);
        let b = v.update(0.0).unwrap();
        // var = 0.5*0.0004 + 0.5*0 = 0.0002 -> vol = sqrt(0.0002)
        assert!((b - (0.0002f64).sqrt()).abs() < 1e-12);
    }

    #[test]
    fn parkinson_warmup() {
        let mut p = Parkinson::new(3);
        assert!(p.update(11.0, 10.0).is_none());
        assert!(p.update(12.0, 10.0).is_none());
        let v = p.update(11.0, 10.0);
        assert!(v.is_some() && v.unwrap() > 0.0);
    }

    #[test]
    fn garman_klass_positive() {
        let mut gk = GarmanKlass::new(2);
        gk.update(100.0, 105.0, 95.0, 102.0);
        let v = gk.update(102.0, 108.0, 99.0, 106.0);
        assert!(v.is_some() && v.unwrap() > 0.0);
    }

    #[test]
    fn garch_positive_variance() {
        let mut g = Garch11::default_params();
        for _ in 0..10 {
            let v = g.update(0.01).unwrap();
            assert!(v > 0.0 && v.is_finite());
        }
    }

    #[test]
    fn rolling_volatility_matches_std() {
        let mut rv = RollingVolatility::new(4);
        let data = [0.01, -0.02, 0.015, 0.005];
        let mut last = None;
        for r in data {
            last = rv.update(r);
        }
        let mean = data.iter().sum::<f64>() / 4.0;
        let var = data.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / 3.0;
        let expected = var.sqrt();
        assert!((last.unwrap() - expected).abs() < 1e-12);
    }
}
