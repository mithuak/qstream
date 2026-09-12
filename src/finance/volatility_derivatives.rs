//! Volatility derivatives (design Phase 7, Tier C).
//!
//! Fleming-Ostdiek-Whaley VIX, Vandermeer VIX, and Demeterfi
//! variance-swap replication. These compute model-free implied volatility
//! and variance-swap rates from rolling windows of returns.

use crate::core::ring::RingBuffer;

/// Fleming-Ostdiek-Whaley VIX Implied Volatility.
///
/// Computes a VIX-style model-free implied volatility index from a rolling
/// window of returns. Uses the variance-swap replication formula:
/// sigma^2 = (2/T) * sum(Delta_K/K^2 * Q(K)) - (1/T)*(F/K0 - 1)^2
/// Simplified here using the realized variance as a proxy.
#[derive(Clone, Debug)]
pub struct FlemingOstdiekWhaleyVIX {
    buf: RingBuffer<f64>,
    annualization: f64,
    update_every: usize,
    since: usize,
}

impl FlemingOstdiekWhaleyVIX {
    /// `window`: rolling window length (e.g. 22 for monthly VIX).
    /// `annualization`: factor to annualize (e.g. 252 for daily data).
    pub fn new(window: usize, annualization: f64, update_every: usize) -> Self {
        assert!(window > 0);
        Self {
            buf: RingBuffer::new(window, 0.0),
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

        let n = self.buf.len();
        let mut data = Vec::with_capacity(n);
        self.buf.fill_vec(&mut data);

        let mut sum_sq = 0.0f64;
        let mut sum_r = 0.0f64;

        for &r in &data {
            sum_sq += r * r;
            sum_r += r;
        }

        let mean_r = sum_r / n as f64;
        let variance = sum_sq / n as f64 - mean_r * mean_r;
        let vix = variance.max(0.0).sqrt() * self.annualization.sqrt();

        Some(vix * 100.0) // In percentage points like VIX
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.since = 0;
    }
}

/// Vandermeer VIX-Implied Volatility Calculator.
///
/// A simplified VIX calculation using the CBOE methodology adapted for
/// streaming returns. Computes the model-free implied variance as the
/// weighted sum of out-of-the-money option payoffs, approximated here
/// from the return distribution.
#[derive(Clone, Debug)]
pub struct VandermeerVIX {
    buf: RingBuffer<f64>,
    annualization: f64,
    update_every: usize,
    since: usize,
}

impl VandermeerVIX {
    pub fn new(window: usize, annualization: f64, update_every: usize) -> Self {
        assert!(window > 0);
        Self {
            buf: RingBuffer::new(window, 0.0),
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

        let n = self.buf.len();
        let mut returns = Vec::with_capacity(n);
        self.buf.fill_vec(&mut returns);
        returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Vandermeer approximation: use interquartile range as volatility proxy
        let q25_idx = n / 4;
        let q75_idx = 3 * n / 4;
        let iqr = returns[q75_idx.min(n - 1)] - returns[q25_idx.min(n - 1)];

        // Scale IQR to standard deviation (for normal: IQR ~ 1.35 sigma)
        let sigma = iqr / 1.35;
        let vix = sigma * self.annualization.sqrt();

        Some(vix * 100.0)
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.since = 0;
    }
}

/// Demeterfi Variance Swap Replication.
///
/// Computes the fair strike of a variance swap using the model-free
/// replication formula. The variance swap rate equals the integral of
/// option prices weighted by 1/K^2, approximated from the return
/// distribution's realized variance plus a correction term.
#[derive(Clone, Debug)]
pub struct DemeterfiVarianceSwap {
    buf: RingBuffer<f64>,
    period: f64,
    annualization: f64,
    update_every: usize,
    since: usize,
}

/// Result from variance swap replication.
#[derive(Clone, Debug)]
pub struct VarianceSwapResult {
    /// Fair variance swap strike (annualized variance).
    pub strike: f64,
    /// Realized variance over the window.
    pub realized_variance: f64,
    /// Convexity correction term.
    pub correction: f64,
}

impl DemeterfiVarianceSwap {
    pub fn new(window: usize, period: f64, annualization: f64, update_every: usize) -> Self {
        assert!(window > 0);
        Self {
            buf: RingBuffer::new(window, 0.0),
            period,
            annualization,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<VarianceSwapResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;

        let n = self.buf.len();
        let mut returns = Vec::with_capacity(n);
        self.buf.fill_vec(&mut returns);

        // Realized variance
        let mut sum_sq = 0.0f64;
        let mut sum_r = 0.0f64;
        for &r in &returns {
            sum_sq += r * r;
            sum_r += r;
        }
        let mean_r = sum_r / n as f64;
        let realized_var = sum_sq / n as f64 - mean_r * mean_r;

        // Convexity correction: E[(ln(S_T/S_0))^2] - (E[ln(S_T/S_0)])^2
        // Approximated from the drift
        let drift = mean_r * n as f64;
        let correction = (drift * drift) / (2.0 * self.period * self.annualization);

        let strike = realized_var * self.annualization + correction;

        Some(VarianceSwapResult {
            strike: strike.max(0.0),
            realized_variance: realized_var * self.annualization,
            correction,
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.since = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fow_vix_runs() {
        let mut vix = FlemingOstdiekWhaleyVIX::new(22, 252.0, 5);
        for i in 0..100 {
            let r = (i as f64 * 0.05).sin() * 0.01;
            if let Some(v) = vix.update(r) {
                assert!(v >= 0.0 && v.is_finite());
            }
        }
    }

    #[test]
    fn vandermeer_vix_runs() {
        let mut vix = VandermeerVIX::new(22, 252.0, 5);
        for i in 0..100 {
            let r = (i as f64 * 0.05).sin() * 0.01;
            if let Some(v) = vix.update(r) {
                assert!(v >= 0.0 && v.is_finite());
            }
        }
    }

    #[test]
    fn demeterfi_var_swap_runs() {
        let mut vs = DemeterfiVarianceSwap::new(22, 30.0 / 365.0, 252.0, 5);
        for i in 0..100 {
            let r = (i as f64 * 0.05).sin() * 0.01;
            if let Some(v) = vs.update(r) {
                assert!(v.strike >= 0.0 && v.strike.is_finite());
            }
        }
    }
}
