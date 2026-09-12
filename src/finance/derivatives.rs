//! Derivatives analytics: Black-Scholes implied volatility (Newton-Raphson),
//! VIX futures term-structure shape, and realized-variance tracking for
//! variance swaps.

use crate::core::matrix::{lu_solve, DMat};
use crate::core::moments::RollingSum;
use crate::finance::risk::{normal_cdf, normal_pdf};

/// Black-Scholes European option price.
pub fn bs_price(spot: f64, strike: f64, ttm: f64, rate: f64, sigma: f64, is_call: bool) -> f64 {
    if ttm <= 0.0 || sigma <= 0.0 {
        let intrinsic = if is_call { (spot - strike).max(0.0) } else { (strike - spot).max(0.0) };
        return intrinsic;
    }
    let sqrt_t = ttm.sqrt();
    let d1 = ((spot / strike).ln() + (rate + 0.5 * sigma * sigma) * ttm) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let disc = (-rate * ttm).exp();
    if is_call {
        spot * normal_cdf(d1) - strike * disc * normal_cdf(d2)
    } else {
        strike * disc * normal_cdf(-d2) - spot * normal_cdf(-d1)
    }
}

/// Black-Scholes vega (dPrice/dSigma), same units as price.
pub fn bs_vega(spot: f64, strike: f64, ttm: f64, rate: f64, sigma: f64) -> f64 {
    if ttm <= 0.0 || sigma <= 0.0 {
        return 0.0;
    }
    let sqrt_t = ttm.sqrt();
    let d1 = ((spot / strike).ln() + (rate + 0.5 * sigma * sigma) * ttm) / (sigma * sqrt_t);
    spot * normal_pdf(d1) * sqrt_t
}

/// Implied volatility solver (Newton-Raphson with bisection fallback).
/// Returns `None` if it fails to converge to a positive volatility.
pub fn implied_volatility(
    market_price: f64,
    spot: f64,
    strike: f64,
    ttm: f64,
    rate: f64,
    is_call: bool,
) -> Option<f64> {
    if spot <= 0.0 || strike <= 0.0 || ttm <= 0.0 || market_price <= 0.0 {
        return None;
    }
    // Initial guess (Manaster-Koehler style).
    let mut sigma = (2.0 * ((spot / strike).ln() + rate * ttm).abs() / ttm).sqrt();
    if !(sigma.is_finite()) || sigma <= 0.0 {
        sigma = 0.2;
    }
    let mut lo = 1e-6f64;
    let mut hi = 10.0f64;
    for _ in 0..100 {
        let price = bs_price(spot, strike, ttm, rate, sigma, is_call);
        let diff = price - market_price;
        if diff.abs() < 1e-8 {
            return Some(sigma);
        }
        // Maintain a bracket for bisection fallback.
        if diff > 0.0 {
            hi = sigma;
        } else {
            lo = sigma;
        }
        let vega = bs_vega(spot, strike, ttm, rate, sigma);
        if vega < 1e-8 {
            // Newton step unstable; bisect.
            sigma = 0.5 * (lo + hi);
        } else {
            let step = sigma - diff / vega;
            sigma = if step > lo && step < hi { step } else { 0.5 * (lo + hi) };
        }
        if !(sigma.is_finite()) || sigma <= 0.0 {
            return None;
        }
    }
    // Return the best estimate if close enough.
    let price = bs_price(spot, strike, ttm, rate, sigma, is_call);
    if (price - market_price).abs() < 1e-4 {
        Some(sigma)
    } else {
        None
    }
}

/// Streaming implied-volatility tracker: each `update` solves IV for the given
/// option snapshot.
#[derive(Clone, Debug)]
pub struct ImpliedVolatility;

impl ImpliedVolatility {
    pub fn new() -> Self {
        Self
    }
    /// Solve IV for one option snapshot.
    pub fn update(
        &mut self,
        market_price: f64,
        spot: f64,
        strike: f64,
        ttm: f64,
        rate: f64,
        is_call: bool,
    ) -> Option<f64> {
        implied_volatility(market_price, spot, strike, ttm, rate, is_call)
    }
    pub fn reset(&mut self) {}
}

impl Default for ImpliedVolatility {
    fn default() -> Self {
        Self::new()
    }
}

/// VIX futures term-structure shape from spot VIX, futures prices and their
/// maturities (in years). Computes a quadratic fit `f(t) = a + b t + c t^2`.
#[derive(Clone, Debug)]
pub struct TermStructure {
    pub slope: f64,
    pub curvature: f64,
    pub contango: f64,
}

/// Compute term-structure slope/curvature/contango for one snapshot.
pub fn term_structure(spot_vix: f64, futures: &[f64], maturities: &[f64]) -> Option<TermStructure> {
    let n = futures.len();
    if n < 2 || n != maturities.len() {
        return None;
    }
    // Least-squares quadratic fit via normal equations (3x3).
    let mut ata = DMat::zeros(3, 3);
    let mut atb = [0.0f64; 3];
    for i in 0..n {
        let t = maturities[i];
        let basis = [1.0, t, t * t];
        for a in 0..3 {
            for b in 0..3 {
                let cur = ata.get(a, b);
                ata.set(a, b, cur + basis[a] * basis[b]);
            }
            atb[a] += basis[a] * futures[i];
        }
    }
    // Regularize slightly for numerical stability when n is small.
    for a in 0..3 {
        let cur = ata.get(a, a);
        ata.set(a, a, cur + 1e-9);
    }
    let coef = match lu_solve(&ata, &atb) {
        Some(c) => c,
        None => return None,
    };
    let slope = coef[1];
    let curvature = 2.0 * coef[2];
    // Contango: near future relative to spot.
    let mut near_idx = 0;
    for i in 1..n {
        if maturities[i] < maturities[near_idx] {
            near_idx = i;
        }
    }
    let contango = if spot_vix.abs() > 1e-12 {
        (futures[near_idx] - spot_vix) / spot_vix
    } else {
        0.0
    };
    Some(TermStructure { slope, curvature, contango })
}

/// Variance-swap realized-variance tracker. Accumulates squared returns over a
/// rolling window; realized variance = sum(r^2), realized vol = sqrt.
#[derive(Clone, Debug)]
pub struct VarianceSwap {
    acc: RollingSum,
}

impl VarianceSwap {
    pub fn new(period: usize) -> Self {
        Self { acc: RollingSum::new(period) }
    }
    pub fn update(&mut self, r: f64) -> Option<f64> {
        self.acc.update(r * r);
        if !self.acc.is_ready() {
            return None;
        }
        Some(self.realized_variance())
    }
    #[inline]
    pub fn realized_variance(&self) -> f64 {
        self.acc.value()
    }
    #[inline]
    pub fn realized_volatility(&self) -> f64 {
        self.acc.value().max(0.0).sqrt()
    }
    /// P&L of a long variance position: `notional * (realized_var - strike_var)`.
    pub fn pnl(&self, strike_variance: f64, notional: f64) -> f64 {
        notional * (self.realized_variance() - strike_variance)
    }
    pub fn reset(&mut self) {
        self.acc.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implied_vol_roundtrip() {
        let sigma_true = 0.25;
        let price = bs_price(100.0, 100.0, 1.0, 0.03, sigma_true, true);
        let iv = implied_volatility(price, 100.0, 100.0, 1.0, 0.03, true).unwrap();
        assert!((iv - sigma_true).abs() < 1e-4);
    }

    #[test]
    fn implied_vol_put_roundtrip() {
        let sigma_true = 0.30;
        let price = bs_price(95.0, 100.0, 0.5, 0.02, sigma_true, false);
        let iv = implied_volatility(price, 95.0, 100.0, 0.5, 0.02, false).unwrap();
        assert!((iv - sigma_true).abs() < 1e-4);
    }

    #[test]
    fn term_structure_contango_positive() {
        let ts = term_structure(20.0, &[21.0, 22.0, 23.0], &[0.1, 0.2, 0.3]).unwrap();
        assert!(ts.contango > 0.0);
        assert!(ts.slope > 0.0);
    }

    #[test]
    fn variance_swap_realized() {
        let mut vs = VarianceSwap::new(3);
        vs.update(0.01);
        vs.update(-0.02);
        let rv = vs.update(0.015).unwrap();
        let expected = 0.01f64 * 0.01 + 0.02 * 0.02 + 0.015 * 0.015;
        assert!((rv - expected).abs() < 1e-15);
    }
}
