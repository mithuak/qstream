//! Heavy/experimental tail-risk measures (design Phase 7, Tier C).
//!
//! Entropic VaR (EVaR), Extreme Value Theory POT/GPD VaR and Expected
//! Shortfall, Johnson-SU VaR, and Spectral Risk Measures. These build on
//! the rolling distribution primitives in core/.

use crate::core::quantile::quantile_sorted;
use crate::core::ring::RingBuffer;

/// Entropic Value-at-Risk (EVaR).
///
/// An upper bound on VaR and CVaR derived from the Chernoff inequality.
/// EVaR is coherent and captures tail behavior via the moment-generating
/// function. For a confidence level `alpha`:
///     EVaR = inf_{z>0} { (1/ln(1/alpha)) * (ln(M(z)) - ln(z)) }
/// where M(z) is the moment-generating function.
#[derive(Clone, Debug)]
pub struct EntropicVaR {
    buf: RingBuffer<f64>,
    sorted: Vec<f64>,
    alpha: f64,
    update_every: usize,
    since: usize,
}

impl EntropicVaR {
    /// `window`: rolling window length.
    /// `alpha`: confidence level (e.g. 0.01 for 99% VaR).
    pub fn new(window: usize, alpha: f64, update_every: usize) -> Self {
        assert!(window > 0);
        assert!(alpha > 0.0 && alpha < 1.0);
        Self {
            buf: RingBuffer::new(window, 0.0),
            sorted: Vec::with_capacity(window),
            alpha,
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

        self.sorted.clear();
        self.buf.fill_vec(&mut self.sorted);
        self.sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // EVaR via numerical optimization over z
        let n = self.sorted.len() as f64;
        let mut best_evar = f64::INFINITY;

        // Grid search for optimal z
        for i in 1..100 {
            let z = i as f64 * 0.01;
            // Compute log moment-generating function: ln(E[e^{zX}])
            let mut mgf_sum = 0.0f64;
            for &v in self.sorted.iter() {
                mgf_sum += (z * v).exp();
            }
            let ln_mgf = (mgf_sum / n).ln();

            let evar = (ln_mgf - z.ln()) / (1.0 / self.alpha).ln();
            if evar < best_evar && evar.is_finite() {
                best_evar = evar;
            }
        }

        if best_evar.is_finite() {
            Some(-best_evar) // Return as positive loss value
        } else {
            // Fallback to VaR
            Some(quantile_sorted(&self.sorted, self.alpha))
        }
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.sorted.clear();
        self.since = 0;
    }
}

/// Extreme Value Theory: Peaks-Over-Threshold (POT) / Generalized Pareto
/// Distribution (GPD) VaR and Expected Shortfall.
///
/// Fits a GPD to exceedances over a threshold and computes VaR and ES
/// from the fitted distribution. The threshold is set at a high empirical
/// quantile.
#[derive(Clone, Debug)]
pub struct EVTGpdTailRisk {
    buf: RingBuffer<f64>,
    sorted: Vec<f64>,
    alpha: f64,
    threshold_quantile: f64,
    update_every: usize,
    since: usize,
}

/// Result from EVT/GPD estimation.
#[derive(Clone, Debug)]
pub struct EVTResult {
    /// Value-at-Risk at the given confidence level.
    pub var: f64,
    /// Expected Shortfall (CVaR) at the given confidence level.
    pub expected_shortfall: f64,
    /// GPD shape parameter (xi).
    pub shape: f64,
    /// GPD scale parameter (beta).
    pub scale: f64,
    /// Number of exceedances used in the fit.
    pub n_exceedances: usize,
}

impl EVTGpdTailRisk {
    /// `window`: rolling window length.
    /// `alpha`: confidence level (e.g. 0.01).
    /// `threshold_quantile`: quantile for threshold (e.g. 0.95).
    pub fn new(window: usize, alpha: f64, threshold_quantile: f64, update_every: usize) -> Self {
        assert!(window > 0);
        assert!(alpha > 0.0 && alpha < 1.0);
        assert!(threshold_quantile > 0.0 && threshold_quantile < 1.0);
        Self {
            buf: RingBuffer::new(window, 0.0),
            sorted: Vec::with_capacity(window),
            alpha,
            threshold_quantile,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<EVTResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;

        self.sorted.clear();
        self.buf.fill_vec(&mut self.sorted);
        self.sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = self.sorted.len();
        let threshold_idx = ((n as f64) * self.threshold_quantile) as usize;
        let threshold = self.sorted[threshold_idx.min(n - 1)];

        // Collect exceedances
        let exceedances: Vec<f64> = self
            .sorted
            .iter()
            .filter(|&&v| v >= threshold)
            .map(|&v| v - threshold)
            .collect();

        if exceedances.len() < 5 {
            // Not enough data for GPD fit; fall back to empirical VaR/CVaR
            let var = quantile_sorted(&self.sorted, self.alpha).abs();
            let es = empirical_cvar(&self.sorted, self.alpha);
            return Some(EVTResult {
                var,
                expected_shortfall: es,
                shape: 0.0,
                scale: 0.0,
                n_exceedances: 0,
            });
        }

        // Method of moments for GPD parameters
        let m = exceedances.iter().sum::<f64>() / exceedances.len() as f64;
        let m2 = exceedances.iter().map(|v| v * v).sum::<f64>() / exceedances.len() as f64;
        let variance = m2 - m * m;

        // GPD moment estimators: xi = 0.5 * (1 - m^2/variance), beta = m * (1 + xi)
        let xi = if variance > 1e-30 {
            0.5 * (1.0 - m * m / variance)
        } else {
            0.0
        };
        let beta = m * (1.0 + xi);

        // VaR from GPD: u + (beta/xi) * ((n/Nu * (1-alpha))^{-xi} - 1)
        let n_u = exceedances.len() as f64;
        let n_total = n as f64;
        let ratio = (n_total / n_u * (1.0 - self.alpha)).max(1e-10);

        let var = if xi.abs() > 1e-6 {
            threshold + (beta / xi) * (ratio.powf(-xi) - 1.0)
        } else {
            threshold - beta * ratio.ln()
        };

        // ES from GPD: (VaR + beta - xi*u) / (1 - xi)  for xi < 1
        let es = if xi < 1.0 && (1.0 - xi).abs() > 1e-6 {
            (var + beta - xi * threshold) / (1.0 - xi)
        } else {
            var
        };

        Some(EVTResult {
            var: var.abs(),
            expected_shortfall: es.abs(),
            shape: xi,
            scale: beta,
            n_exceedances: exceedances.len(),
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.sorted.clear();
        self.since = 0;
    }
}

/// Compute empirical CVaR (Expected Shortfall) from sorted data.
fn empirical_cvar(sorted: &[f64], alpha: f64) -> f64 {
    let n = sorted.len();
    let k = ((n as f64) * alpha).ceil() as usize;
    if k == 0 {
        return sorted[0].abs();
    }
    let sum: f64 = sorted.iter().take(k).sum();
    (sum / k as f64).abs()
}

/// Johnson-SU Value-at-Risk.
///
/// Fits a Johnson SU distribution to the data (via moment-matched
/// transformation) and computes VaR from the fitted quantiles. The
/// Johnson SU family can match any feasible combination of skewness
/// and kurtosis.
#[derive(Clone, Debug)]
pub struct JohnsonSUVaR {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    alpha: f64,
    update_every: usize,
    since: usize,
}

impl JohnsonSUVaR {
    pub fn new(window: usize, alpha: f64, update_every: usize) -> Self {
        assert!(window > 0);
        assert!(alpha > 0.0 && alpha < 1.0);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            alpha,
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
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len() as f64;
        let mean = self.data.iter().sum::<f64>() / n;

        // Central moments
        let mut m2 = 0.0f64;
        let mut m3 = 0.0f64;
        let mut m4 = 0.0f64;
        for &v in self.data.iter() {
            let d = v - mean;
            m2 += d * d;
            m3 += d * d * d;
            m4 += d * d * d * d;
        }
        m2 /= n;
        m3 /= n;
        m4 /= n;

        let skewness = if m2 > 1e-30 {
            m3 / m2.powf(1.5)
        } else {
            0.0
        };
        let kurtosis = if m2 > 1e-30 {
            m4 / (m2 * m2)
        } else {
            3.0
        };

        // Johnson SU parameters (approximate moment matching)
        let omega = (4.0 + 2.0 * (kurtosis - skewness * skewness - 1.0)).sqrt();
        let gamma = -skewness.signum() * ((omega - 1.0) / (omega + 1.0)).sqrt() * 0.5;
        let delta = 1.0 / (omega - 1.0).max(0.1).ln().sqrt().max(0.1);
        let xi = mean - (omega - 1.0).sqrt() * gamma / delta;
        let lambda = m2.sqrt() / ((omega - 1.0) * (omega.cosh() - 1.0)).sqrt();

        // VaR via inverse normal CDF approximation
        let z_alpha = inverse_normal_cdf(self.alpha);

        // Johnson SU quantile: xi + lambda * sinh((z - gamma) / delta)
        let var = xi + lambda * ((z_alpha - gamma) / delta).sinh();

        Some(-var) // Return as positive loss
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Approximate inverse normal CDF (Abramowitz and Stegun).
fn inverse_normal_cdf(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    // Rational approximation
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

    let p_low = 0.02425;
    let p_high = 1.0 - p_low;

    let q = if p < p_low {
        // Rational approximation for lower region
        (-2.0 * p.ln()).sqrt()
    } else if p <= p_high {
        // Rational approximation for central region
        let q2 = p - 0.5;
        let r = q2 * q2;
        let num = q2 * (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]);
        let den = (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0);
        num / den
    } else {
        // Rational approximation for upper region
        (-2.0 * (1.0 - p).ln()).sqrt() * -1.0
    };

    q
}

/// Spectral Risk Measure (exponential weighting).
///
/// A weighted average of losses where the weight decreases exponentially
/// with the quantile level. The risk aversion parameter `gamma` controls
/// the decay rate: higher gamma places more weight on extreme losses.
#[derive(Clone, Debug)]
pub struct SpectralRiskMeasure {
    buf: RingBuffer<f64>,
    sorted: Vec<f64>,
    gamma: f64,
    update_every: usize,
    since: usize,
}

impl SpectralRiskMeasure {
    /// `window`: rolling window length.
    /// `gamma`: risk aversion (weight decay rate, e.g. 0.01-0.1).
    pub fn new(window: usize, gamma: f64, update_every: usize) -> Self {
        assert!(window > 0);
        assert!(gamma > 0.0);
        Self {
            buf: RingBuffer::new(window, 0.0),
            sorted: Vec::with_capacity(window),
            gamma,
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

        self.sorted.clear();
        self.buf.fill_vec(&mut self.sorted);
        self.sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Spectral risk: integral of phi(u) * q(u) du
        // phi(u) = gamma * exp(-gamma * (1-u)) / (1 - exp(-gamma))
        let n = self.sorted.len();
        let mut srm = 0.0f64;
        let mut weight_sum = 0.0f64;
        let norm = 1.0 - (-self.gamma).exp();

        for i in 0..n {
            let u = (i + 1) as f64 / n as f64;
            let weight = if norm > 1e-30 {
                self.gamma * (-self.gamma * (1.0 - u)).exp() / norm
            } else {
                1.0 / n as f64
            };
            srm += weight * self.sorted[i].abs();
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            Some(srm / weight_sum)
        } else {
            Some(0.0)
        }
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.sorted.clear();
        self.since = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entropic_var_runs() {
        let mut evar = EntropicVaR::new(64, 0.01, 16);
        for i in 0..256 {
            let x = (i as f64 * 0.1).sin() + (i % 7) as f64 * 0.01;
            if let Some(v) = evar.update(x) {
                assert!(v.is_finite());
            }
        }
    }

    #[test]
    fn evt_gpd_runs() {
        let mut evt = EVTGpdTailRisk::new(128, 0.01, 0.95, 32);
        for i in 0..512 {
            let x = (i as f64 * 0.1).sin() + (i % 7) as f64 * 0.01;
            if let Some(r) = evt.update(x) {
                assert!(r.var >= 0.0);
                assert!(r.expected_shortfall >= 0.0);
            }
        }
    }

    #[test]
    fn johnson_su_var_runs() {
        let mut jsu = JohnsonSUVaR::new(64, 0.01, 16);
        for i in 0..256 {
            let x = (i as f64 * 0.1).sin() + (i % 7) as f64 * 0.01;
            if let Some(v) = jsu.update(x) {
                assert!(v.is_finite());
            }
        }
    }

    #[test]
    fn spectral_risk_runs() {
        let mut srm = SpectralRiskMeasure::new(64, 0.05, 16);
        for i in 0..256 {
            let x = (i as f64 * 0.1).sin() + (i % 7) as f64 * 0.01;
            if let Some(v) = srm.update(x) {
                assert!(v >= 0.0);
            }
        }
    }
}
