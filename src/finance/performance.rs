//! Performance / risk-adjusted return metrics: Sharpe, Sortino (with optional
//! MAR), up/down capture, gain-loss (Bernardo-Ledoit), Omega, conditional
//! Sharpe, and the Deflated Sharpe Ratio (Bailey & Lopez de Prado).

use crate::core::covariance::RollingCovariance;
use crate::core::moments::{RollingMean, RollingMoments, RollingSum, RollingVariance};
use crate::core::ring::RingBuffer;
use crate::finance::risk::normal_cdf;

const EULER_MASCHERONI: f64 = 0.5772156649015329;

/// Rolling Sharpe ratio `(mean - rf) / std`.
#[derive(Clone, Debug)]
pub struct SharpeRatio {
    moments: RollingMoments,
    rf: f64,
}

impl SharpeRatio {
    pub fn new(period: usize, rf: f64) -> Self {
        Self { moments: RollingMoments::new(period), rf }
    }
    pub fn update(&mut self, r: f64) -> Option<f64> {
        self.moments.update(r);
        if !self.moments.is_ready() {
            return None;
        }
        let sd = self.moments.std_dev();
        if sd <= 1e-18 {
            return None;
        }
        Some((self.moments.mean() - self.rf) / sd)
    }
    pub fn reset(&mut self) {
        self.moments.reset();
    }
}

/// Sortino ratio with a target / minimum acceptable return (MAR).
/// `(mean - mar) / downside_deviation`.
#[derive(Clone, Debug)]
pub struct SortinoRatio {
    mean: RollingMean,
    downside: RollingMean,
    mar: f64,
}

impl SortinoRatio {
    pub fn new(period: usize, mar: f64) -> Self {
        Self { mean: RollingMean::new(period), downside: RollingMean::new(period), mar }
    }
    pub fn update(&mut self, r: f64) -> Option<f64> {
        self.mean.update(r);
        let d = (r - self.mar).min(0.0);
        self.downside.update(d * d);
        if !self.mean.is_ready() {
            return None;
        }
        let dd = self.downside.value().max(0.0).sqrt();
        if dd <= 1e-18 {
            return None;
        }
        Some((self.mean.value() - self.mar) / dd)
    }
    #[inline]
    pub fn downside_deviation(&self) -> f64 {
        self.downside.value().max(0.0).sqrt()
    }
    pub fn reset(&mut self) {
        self.mean.reset();
        self.downside.reset();
    }
}

/// Gain-loss ratio (Bernardo-Ledoit): sum of gains / sum of loss magnitudes.
#[derive(Clone, Debug)]
pub struct GainLossRatio {
    gains: RollingSum,
    losses: RollingSum,
}

impl GainLossRatio {
    pub fn new(period: usize) -> Self {
        Self { gains: RollingSum::new(period), losses: RollingSum::new(period) }
    }
    pub fn update(&mut self, r: f64) -> Option<f64> {
        self.gains.update(r.max(0.0));
        self.losses.update((-r).max(0.0));
        if !self.gains.is_ready() {
            return None;
        }
        let l = self.losses.value();
        if l <= 1e-18 {
            return None;
        }
        Some(self.gains.value() / l)
    }
    pub fn reset(&mut self) {
        self.gains.reset();
        self.losses.reset();
    }
}

/// Omega ratio: probability-weighted gains over losses above a threshold.
/// `sum(max(r-thr,0)) / sum(max(thr-r,0))`.
#[derive(Clone, Debug)]
pub struct OmegaRatio {
    gains: RollingSum,
    losses: RollingSum,
    threshold: f64,
}

impl OmegaRatio {
    pub fn new(period: usize, threshold: f64) -> Self {
        Self { gains: RollingSum::new(period), losses: RollingSum::new(period), threshold }
    }
    pub fn update(&mut self, r: f64) -> Option<f64> {
        self.gains.update((r - self.threshold).max(0.0));
        self.losses.update((self.threshold - r).max(0.0));
        if !self.gains.is_ready() {
            return None;
        }
        let l = self.losses.value();
        if l <= 1e-18 {
            return None;
        }
        Some(self.gains.value() / l)
    }
    pub fn reset(&mut self) {
        self.gains.reset();
        self.losses.reset();
    }
}

/// Upside/downside capture ratios versus a benchmark over a rolling window.
/// Computed on demand from the retained pair window (Tier B).
#[derive(Clone, Debug)]
pub struct CaptureRatios {
    buf: RingBuffer<(f64, f64)>,
}

impl CaptureRatios {
    pub fn new(period: usize) -> Self {
        Self { buf: RingBuffer::new(period, (0.0, 0.0)) }
    }
    pub fn update(&mut self, asset: f64, benchmark: f64) -> Option<(f64, f64)> {
        self.buf.push((asset, benchmark));
        if !self.buf.is_full() {
            return None;
        }
        Some((self.up_capture(), self.down_capture()))
    }
    fn capture(&self, up: bool) -> f64 {
        let (mut sa, mut sb, mut n) = (0.0, 0.0, 0usize);
        for i in 0..self.buf.len() {
            let (a, b) = *self.buf.get(i);
            if (up && b > 0.0) || (!up && b < 0.0) {
                sa += a;
                sb += b;
                n += 1;
            }
        }
        if n == 0 || sb.abs() < 1e-18 {
            return f64::NAN;
        }
        (sa / n as f64) / (sb / n as f64)
    }
    pub fn up_capture(&self) -> f64 {
        self.capture(true)
    }
    pub fn down_capture(&self) -> f64 {
        self.capture(false)
    }
    pub fn reset(&mut self) {
        self.buf.clear((0.0, 0.0));
    }
}

/// Conditional Sharpe ratio: Sharpe of the asset conditioned on up vs down
/// benchmark states, over a rolling window.
#[derive(Clone, Debug)]
pub struct ConditionalSharpe {
    buf: RingBuffer<(f64, f64)>,
    scratch: Vec<f64>,
}

impl ConditionalSharpe {
    pub fn new(period: usize) -> Self {
        Self { buf: RingBuffer::new(period, (0.0, 0.0)), scratch: Vec::with_capacity(period) }
    }
    pub fn update(&mut self, asset: f64, benchmark: f64) {
        self.buf.push((asset, benchmark));
    }
    fn sharpe_subset(&mut self, up: bool) -> f64 {
        self.scratch.clear();
        for i in 0..self.buf.len() {
            let (a, b) = *self.buf.get(i);
            if (up && b > 0.0) || (!up && b < 0.0) {
                self.scratch.push(a);
            }
        }
        let n = self.scratch.len() as f64;
        if n < 2.0 {
            return f64::NAN;
        }
        let mean = self.scratch.iter().sum::<f64>() / n;
        let var = self.scratch.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let sd = var.sqrt();
        if sd <= 1e-18 {
            return f64::NAN;
        }
        mean / sd
    }
    pub fn up_sharpe(&mut self) -> f64 {
        self.sharpe_subset(true)
    }
    pub fn down_sharpe(&mut self) -> f64 {
        self.sharpe_subset(false)
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.buf.is_full()
    }
    pub fn reset(&mut self) {
        self.buf.clear((0.0, 0.0));
        self.scratch.clear();
    }
}

/// Deflated Sharpe Ratio (Bailey & Lopez de Prado, 2014). Adjusts the observed
/// Sharpe for the number of independent trials (selection bias) and for the
/// non-normality (skew/kurtosis) of returns.
#[derive(Clone, Debug)]
pub struct DeflatedSharpeRatio {
    moments: RollingMoments,
    n_trials: usize,
    rf: f64,
}

impl DeflatedSharpeRatio {
    /// `n_trials` is the number of independent strategy trials considered.
    pub fn new(period: usize, n_trials: usize, rf: f64) -> Self {
        assert!(n_trials >= 1, "n_trials must be >= 1");
        Self { moments: RollingMoments::new(period), n_trials, rf }
    }
    pub fn update(&mut self, r: f64) -> Option<f64> {
        self.moments.update(r);
        if !self.moments.is_ready() {
            return None;
        }
        Some(self.value())
    }
    /// Current deflated Sharpe ratio (probability that the observed Sharpe is
    /// greater than the expected maximum Sharpe under the null of `n_trials`).
    pub fn value(&self) -> f64 {
        let t = self.moments.count() as f64;
        if t < 3.0 {
            return f64::NAN;
        }
        let sd = self.moments.std_dev();
        if sd <= 1e-18 {
            return f64::NAN;
        }
        let sr = (self.moments.mean() - self.rf) / sd;
        let skew = self.moments.skewness();
        let kurt = self.moments.kurtosis(); // non-excess
        // Variance of the Sharpe ratio estimate (Lo, 2002 / Bailey-LdP).
        let var_sr = (1.0 - skew * sr + ((kurt - 1.0) / 4.0) * sr * sr) / (t - 1.0);
        if var_sr <= 0.0 {
            return f64::NAN;
        }
        let sd_sr = var_sr.sqrt();
        // Expected maximum Sharpe under the null of n_trials independent trials.
        let n = self.n_trials as f64;
        let z1 = inv_norm_cdf(1.0 - 1.0 / n);
        let z2 = inv_norm_cdf(1.0 - 1.0 / (n * std::f64::consts::E));
        let sr0 = sd_sr * ((1.0 - EULER_MASCHERONI) * z1 + EULER_MASCHERONI * z2);
        normal_cdf((sr - sr0) / sd_sr)
    }
    pub fn reset(&mut self) {
        self.moments.reset();
    }
}

/// Inverse standard normal CDF used by the deflated Sharpe (delegates to the
/// finance risk module's `probit`).
#[inline]
fn inv_norm_cdf(p: f64) -> f64 {
    crate::finance::risk::probit(p)
}

/// Lo (2002) autocorrelation-adjusted Sharpe ratio. Corrects the observed
/// Sharpe for serial correlation in returns via the AR(1) variance-ratio
/// factor: `SR_adj = SR * sqrt((1 - rho) / (1 + rho))`, where `rho` is the
/// rolling lag-1 autocorrelation.
#[derive(Clone, Debug)]
pub struct LoAutocorrelationSharpe {
    moments: RollingMoments,
    cov: RollingCovariance,
    varr: RollingVariance,
    prev: f64,
    have_prev: bool,
    rf: f64,
}

impl LoAutocorrelationSharpe {
    pub fn new(period: usize, rf: f64) -> Self {
        Self {
            moments: RollingMoments::new(period),
            cov: RollingCovariance::new(period),
            varr: RollingVariance::new(period),
            prev: 0.0,
            have_prev: false,
            rf,
        }
    }
    pub fn update(&mut self, r: f64) -> Option<f64> {
        self.moments.update(r);
        if self.have_prev {
            self.cov.update(r, self.prev);
            self.varr.update(r);
        }
        self.prev = r;
        self.have_prev = true;
        if !self.moments.is_ready() || !self.cov.is_ready() {
            return None;
        }
        let sd = self.moments.std_dev();
        if sd <= 1e-18 {
            return None;
        }
        let sr = (self.moments.mean() - self.rf) / sd;
        let var = self.varr.variance();
        let rho = if var > 1e-18 {
            (self.cov.covariance() / var).clamp(-0.999, 0.999)
        } else {
            0.0
        };
        let factor = ((1.0 - rho) / (1.0 + rho)).max(0.0).sqrt();
        Some(sr * factor)
    }
    #[inline]
    pub fn autocorrelation(&self) -> f64 {
        let var = self.varr.variance();
        if var > 1e-18 {
            (self.cov.covariance() / var).clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }
    pub fn reset(&mut self) {
        self.moments.reset();
        self.cov.reset();
        self.varr.reset();
        self.prev = 0.0;
        self.have_prev = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sortino_positive_for_upside() {
        let mut s = SortinoRatio::new(5, 0.0);
        let mut out = None;
        for r in [0.02, 0.01, -0.005, 0.03, 0.01] {
            out = s.update(r);
        }
        assert!(out.unwrap() > 0.0);
    }

    #[test]
    fn gain_loss_ratio_basic() {
        let mut g = GainLossRatio::new(4);
        g.update(0.02);
        g.update(-0.01);
        g.update(0.03);
        let v = g.update(-0.01).unwrap();
        // gains 0.05, losses 0.02 -> 2.5
        assert!((v - 2.5).abs() < 1e-9);
    }

    #[test]
    fn capture_up_when_scaled() {
        let mut c = CaptureRatios::new(4);
        c.update(0.02, 0.01);
        c.update(-0.01, -0.005);
        c.update(0.04, 0.02);
        let (up, down) = c.update(-0.02, -0.01).unwrap();
        assert!((up - 2.0).abs() < 1e-9);
        assert!((down - 2.0).abs() < 1e-9);
    }

    #[test]
    fn deflated_sharpe_in_unit_interval() {
        let mut d = DeflatedSharpeRatio::new(30, 5, 0.0);
        let mut out = None;
        for i in 0..40 {
            let r = ((i % 5) as f64 - 2.0) * 0.01;
            out = d.update(r);
        }
        let v = out.unwrap();
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn lo_sharpe_positive_autocorr_deflates() {
        // AR(1) with rho ~ 0.9 driven by pseudo-white noise (LCG) plus drift,
        // so the rolling lag-1 autocorrelation is reliably positive.
        let mut seed: u64 = 0x9E3779B97F4A7C15;
        let mut noise = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((seed >> 33) as f64) / (1u64 << 31) as f64 - 0.5
        };
        let mut lo = LoAutocorrelationSharpe::new(50, 0.0);
        let mut sr = SharpeRatio::new(50, 0.0);
        let mut prev = 0.0;
        let mut adj = None;
        let mut raw = None;
        for _ in 0..400 {
            let r = 0.9 * prev + 0.002 + noise() * 0.001;
            prev = r;
            adj = lo.update(r);
            raw = sr.update(r);
        }
        assert!(lo.autocorrelation() > 0.3, "rho = {}", lo.autocorrelation());
        // Positive autocorrelation => factor < 1 => |adjusted| < |raw|.
        assert!(adj.unwrap().abs() < raw.unwrap().abs());
    }
}
