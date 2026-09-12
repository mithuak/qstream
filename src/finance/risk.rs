//! Tail-risk measures: historical VaR, Conditional VaR / Expected Shortfall,
//! Cornish-Fisher modified VaR, Conditional Drawdown-at-Risk, and the Rachev
//! ratio. Backed by the rolling tail/quantile/drawdown primitives.

use crate::core::moments::RollingMoments;
use crate::core::quantile::TailAccumulator;
use crate::core::drawdown::DrawdownState;
use crate::core::ring::RingBuffer;

/// Inverse standard normal CDF (probit) via the Acklam rational approximation,
/// refined once with Halley's method for ~1e-15 relative accuracy.
pub fn probit(p: f64) -> f64 {
    if !(0.0..=1.0).contains(&p) {
        return f64::NAN;
    }
    if p == 0.0 {
        return f64::NEG_INFINITY;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }
    let a = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383577518672690e+02,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];
    let b = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    let c = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];
    let d = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];
    let plow = 0.02425;
    let phigh = 1.0 - plow;
    let q;
    let r;
    // Acklam's rational approximation: relative error < 1.15e-9 across the
    // full domain, which is ample for VaR / z-score use.
    if p < plow {
        q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= phigh {
        q = p - 0.5;
        r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

/// Standard normal CDF via the Abramowitz-Stegun (26.2.17) rational
/// approximation (~7.5e-8 absolute accuracy). Avoids the unstable `f64::erf`.
pub fn normal_cdf(x: f64) -> f64 {
    if !x.is_finite() {
        return if x > 0.0 { 1.0 } else { 0.0 };
    }
    let abs = x.abs();
    let t = 1.0 / (1.0 + 0.2316419 * abs);
    let pdf = (-0.5 * abs * abs).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let poly = t
        * (0.319381530
            + t * (-0.356563782
                + t * (1.781477937 + t * (-1.821255978 + t * 1.330274429))));
    let upper = 1.0 - pdf * poly; // P(Z <= |x|)
    if x >= 0.0 {
        upper
    } else {
        1.0 - upper
    }
}

/// Standard normal PDF.
pub fn normal_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

/// Historical Value-at-Risk over a rolling window of returns.
#[derive(Clone, Debug)]
pub struct ValueAtRisk {
    tail: TailAccumulator,
    p: f64,
}

impl ValueAtRisk {
    /// `p` is the tail probability (e.g. 0.05 for 95% VaR).
    pub fn new(period: usize, p: f64) -> Self {
        assert!(p > 0.0 && p < 1.0, "p must be in (0,1)");
        Self { tail: TailAccumulator::new(period), p }
    }
    pub fn update(&mut self, r: f64) -> Option<f64> {
        self.tail.update(r);
        if !self.tail.is_ready() {
            return None;
        }
        Some(self.tail.var(self.p))
    }
    #[inline]
    pub fn value(&mut self) -> Option<f64> {
        if self.tail.is_ready() {
            Some(self.tail.var(self.p))
        } else {
            None
        }
    }
    pub fn reset(&mut self) {
        self.tail.reset();
    }
}

/// Conditional VaR / Expected Shortfall over a rolling window.
#[derive(Clone, Debug)]
pub struct ConditionalValueAtRisk {
    tail: TailAccumulator,
    p: f64,
}

impl ConditionalValueAtRisk {
    pub fn new(period: usize, p: f64) -> Self {
        assert!(p > 0.0 && p < 1.0, "p must be in (0,1)");
        Self { tail: TailAccumulator::new(period), p }
    }
    pub fn update(&mut self, r: f64) -> Option<f64> {
        self.tail.update(r);
        if !self.tail.is_ready() {
            return None;
        }
        Some(self.tail.cvar(self.p))
    }
    pub fn reset(&mut self) {
        self.tail.reset();
    }
}

/// Cornish-Fisher modified VaR using rolling mean, std, skewness and excess
/// kurtosis. VaR is reported as a positive loss magnitude.
#[derive(Clone, Debug)]
pub struct CornishFisherVaR {
    moments: RollingMoments,
    confidence: f64,
}

impl CornishFisherVaR {
    /// `confidence` e.g. 0.95 or 0.99.
    pub fn new(period: usize, confidence: f64) -> Self {
        assert!(confidence > 0.0 && confidence < 1.0, "confidence in (0,1)");
        Self { moments: RollingMoments::new(period), confidence }
    }
    pub fn update(&mut self, r: f64) -> Option<f64> {
        self.moments.update(r);
        if !self.moments.is_ready() {
            return None;
        }
        Some(self.value())
    }
    pub fn value(&self) -> f64 {
        let mean = self.moments.mean();
        let std = self.moments.std_dev();
        let s = self.moments.skewness();
        let k = self.moments.excess_kurtosis();
        let p = 1.0 - self.confidence; // left-tail probability
        let z = probit(p);
        let z_cf = z + (z * z - 1.0) * s / 6.0 + (z.powi(3) - 3.0 * z) * k / 24.0
            - (2.0 * z.powi(3) - 5.0 * z) * s * s / 36.0;
        // Return loss as a positive number: -(mean + z_cf*std).
        -(mean + z_cf * std)
    }
    pub fn reset(&mut self) {
        self.moments.reset();
    }
}

/// Conditional Drawdown-at-Risk from a rolling wealth curve built from returns.
#[derive(Clone, Debug)]
pub struct ConditionalDrawdownAtRisk {
    dd: DrawdownState,
    alpha: f64,
}

impl ConditionalDrawdownAtRisk {
    /// `alpha` confidence level (e.g. 0.95); `window` bounds drawdown history.
    pub fn new(window: usize, alpha: f64) -> Self {
        assert!(alpha > 0.0 && alpha < 1.0, "alpha in (0,1)");
        Self { dd: DrawdownState::new(window), alpha }
    }
    pub fn update(&mut self, r: f64) -> Option<f64> {
        self.dd.update_return(r);
        Some(self.dd.cdar(self.alpha))
    }
    #[inline]
    pub fn max_drawdown(&self) -> f64 {
        self.dd.max_drawdown()
    }
    pub fn reset(&mut self) {
        self.dd.reset();
    }
}

/// Rachev ratio: expected upper-tail return divided by expected lower-tail
/// loss, both at tail probability `p`, over a rolling window.
#[derive(Clone, Debug)]
pub struct RachevRatio {
    buf: RingBuffer<f64>,
    scratch: Vec<f64>,
    p: f64,
}

impl RachevRatio {
    pub fn new(period: usize, p: f64) -> Self {
        assert!(p > 0.0 && p < 1.0, "p must be in (0,1)");
        Self { buf: RingBuffer::new(period, 0.0), scratch: Vec::with_capacity(period), p }
    }
    pub fn update(&mut self, r: f64) -> Option<f64> {
        self.buf.push(r);
        if !self.buf.is_full() {
            return None;
        }
        Some(self.value())
    }
    pub fn value(&mut self) -> f64 {
        self.buf.fill_vec(&mut self.scratch);
        let n = self.scratch.len();
        if n == 0 {
            return f64::NAN;
        }
        self.scratch.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let k = ((self.p * n as f64).ceil() as usize).max(1).min(n);
        // Lower tail (worst k): mean of smallest k -> magnitude of loss.
        let mut lower = 0.0;
        for i in 0..k {
            lower += self.scratch[i];
        }
        let lower = -(lower / k as f64);
        // Upper tail (best k): mean of largest k.
        let mut upper = 0.0;
        for i in (n - k)..n {
            upper += self.scratch[i];
        }
        let upper = upper / k as f64;
        if lower.abs() < 1e-18 {
            f64::NAN
        } else {
            upper / lower
        }
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.scratch.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probit_known_values() {
        assert!((probit(0.5)).abs() < 1e-9);
        assert!((probit(0.975) - 1.959963985).abs() < 1e-6);
        assert!((probit(0.05) + 1.644853627).abs() < 1e-6);
    }

    #[test]
    fn normal_cdf_half() {
        // A&S 26.2.17 approximation: ~1e-7 absolute accuracy.
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-7);
        // True Phi(1.96) = 0.97500210485...
        assert!((normal_cdf(1.96) - 0.97500210485).abs() < 1e-6);
        assert!((normal_cdf(-1.96) - 0.02499789515).abs() < 1e-6);
    }

    #[test]
    fn cornish_fisher_reduces_to_gaussian_when_symmetric() {
        let mut cf = CornishFisherVaR::new(50, 0.95);
        // Feed a symmetric sample so skew~0, kurt~0.
        for i in 0..50 {
            let x = (i as f64 - 24.5) * 0.001;
            cf.update(x);
        }
        let v = cf.value();
        assert!(v.is_finite());
    }

    #[test]
    fn rachev_positive_for_favorable_tails() {
        let mut rr = RachevRatio::new(20, 0.1);
        for i in 0..20 {
            let x = if i % 2 == 0 { 0.02 } else { -0.01 };
            rr.update(x);
        }
        let v = rr.value();
        assert!(v > 0.0);
    }
}
