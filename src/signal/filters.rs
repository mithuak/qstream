//! Smoothing filters: Savitzky-Golay (precomputed convolution coefficients),
//! Kolmogorov-Zurbenko (cascaded moving averages), and a local (Wiener2-style)
//! adaptive denoiser.

use crate::core::matrix::{lu_solve, DMat};
use crate::core::moments::{RollingMean, RollingVariance};
use crate::core::ring::RingBuffer;
use crate::core::traits::ScalarIndicator;

/// Precompute Savitzky-Golay smoothing coefficients for a centered evaluation
/// point (target offset 0). `window` must be odd and greater than `order`.
fn savgol_coeffs(window: usize, order: usize) -> Vec<f64> {
    let m = (window - 1) / 2;
    let n = order + 1;
    let mut a = DMat::zeros(window, n);
    for i in 0..window {
        let off = i as f64 - m as f64;
        for j in 0..n {
            a.set(i, j, off.powi(j as i32));
        }
    }
    let at = a.transpose();
    let ata = at.mul(&a);
    let mut e0 = vec![0.0; n];
    e0[0] = 1.0;
    let z = lu_solve(&ata, &e0).expect("Savitzky-Golay normal matrix is singular");
    a.mul_vec(&z)
}

/// Savitzky-Golay polynomial smoothing over a rolling window. Coefficients are
/// computed once in the constructor; each update is a fixed-length dot product
/// (Tier B). Output corresponds to the window center (group delay `window/2`).
#[derive(Clone, Debug)]
pub struct SavitzkyGolay {
    buf: RingBuffer<f64>,
    coeffs: Vec<f64>,
}

impl SavitzkyGolay {
    pub fn new(window: usize, order: usize) -> Self {
        assert!(window % 2 == 1, "window must be odd");
        assert!(window > order, "window must exceed polynomial order");
        let coeffs = savgol_coeffs(window, order);
        Self { buf: RingBuffer::new(window, 0.0), coeffs }
    }
    /// Derivative estimation of given `deriv` order at the window center.
    pub fn with_derivative(window: usize, order: usize, deriv: usize) -> Self {
        assert!(window % 2 == 1, "window must be odd");
        assert!(window > order, "window must exceed polynomial order");
        assert!(deriv <= order, "derivative order must be <= polynomial order");
        let coeffs = savgol_coeffs_deriv(window, order, deriv);
        Self { buf: RingBuffer::new(window, 0.0), coeffs }
    }
}

fn savgol_coeffs_deriv(window: usize, order: usize, deriv: usize) -> Vec<f64> {
    let m = (window - 1) / 2;
    let n = order + 1;
    let mut a = DMat::zeros(window, n);
    for i in 0..window {
        let off = i as f64 - m as f64;
        for j in 0..n {
            a.set(i, j, off.powi(j as i32));
        }
    }
    let at = a.transpose();
    let ata = at.mul(&a);
    // Target vector: derivative `deriv` of t^j at t=0 is j!/(j-deriv)! for
    // j == deriv, else 0 (evaluated at t=0 only the j==deriv term survives).
    let mut v = vec![0.0; n];
    let mut fact = 1.0f64;
    for k in 1..=deriv {
        fact *= k as f64;
    }
    v[deriv] = fact;
    let z = lu_solve(&ata, &v).expect("Savitzky-Golay normal matrix is singular");
    a.mul_vec(&z)
}

impl ScalarIndicator for SavitzkyGolay {
    type Output = f64;
    fn update(&mut self, value: f64) -> Option<f64> {
        self.buf.push(value);
        if !self.buf.is_full() {
            return None;
        }
        let mut acc = 0.0;
        for i in 0..self.buf.len() {
            acc += self.coeffs[i] * self.buf.get(i);
        }
        Some(acc)
    }
    fn reset(&mut self) {
        self.buf.clear(0.0);
    }
}

/// Kolmogorov-Zurbenko filter: `passes` cascaded moving averages of length
/// `window`. Strong low-pass with near-Gaussian transfer.
#[derive(Clone, Debug)]
pub struct KolmogorovZurbenko {
    stages: Vec<RollingMean>,
}

impl KolmogorovZurbenko {
    pub fn new(window: usize, passes: usize) -> Self {
        assert!(window > 0 && passes > 0, "window and passes must be > 0");
        let stages = (0..passes).map(|_| RollingMean::new(window)).collect();
        Self { stages }
    }
}

impl ScalarIndicator for KolmogorovZurbenko {
    type Output = f64;
    fn update(&mut self, value: f64) -> Option<f64> {
        let mut x = value;
        let mut ready = true;
        for stage in self.stages.iter_mut() {
            x = stage.update(x);
            if !stage.is_ready() {
                ready = false;
            }
        }
        if ready {
            Some(x)
        } else {
            None
        }
    }
    fn reset(&mut self) {
        for stage in self.stages.iter_mut() {
            stage.reset();
        }
    }
}

/// Local (Wiener2-style) adaptive denoiser. Over a rolling window it estimates
/// the local mean and variance, tracks a noise-variance baseline, and shrinks
/// the deviation from the local mean by the Wiener gain
/// `max(local_var - noise_var, 0) / local_var`.
#[derive(Clone, Debug)]
pub struct WienerFilter {
    mean: RollingMean,
    var: RollingVariance,
    noise: RollingMean,
}

impl WienerFilter {
    /// `window` local estimation window; `noise_window` baseline averaging.
    pub fn new(window: usize, noise_window: usize) -> Self {
        Self {
            mean: RollingMean::new(window),
            var: RollingVariance::new(window),
            noise: RollingMean::new(noise_window),
        }
    }
}

impl ScalarIndicator for WienerFilter {
    type Output = f64;
    fn update(&mut self, value: f64) -> Option<f64> {
        self.mean.update(value);
        let local_var = self.var.update(value);
        if !self.mean.is_ready() {
            return None;
        }
        // Baseline noise variance: average of local variances.
        let noise_var = self.noise.update(local_var);
        let mu = self.mean.value();
        let gain = if local_var > 1e-18 {
            ((local_var - noise_var).max(0.0)) / local_var
        } else {
            0.0
        };
        Some(mu + gain * (value - mu))
    }
    fn reset(&mut self) {
        self.mean.reset();
        self.var.reset();
        self.noise.reset();
    }
}

/// Frequency estimate from an adaptive-cycle filter.
#[derive(Clone, Copy, Debug)]
pub struct FrequencyEstimate {
    /// Estimated normalized frequency in cycles/sample (0..0.5).
    pub frequency: f64,
    /// Notched (interference-removed) output for the current sample.
    pub filtered: f64,
}

/// Adaptive notch filter frequency estimator: a second-order IIR notch whose
/// center frequency `omega` is adapted by gradient descent to minimize output
/// power (so it locks onto and removes a dominant sinusoid). A sensitivity
/// filter provides `d(output)/d(omega)`. O(1) recursive, Tier A.
#[derive(Clone, Debug)]
pub struct AdaptiveNotchFilter {
    rho: f64,
    mu: f64,
    omega: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
    d1: f64,
    d2: f64,
}

impl AdaptiveNotchFilter {
    /// `rho` pole radius in (0,1) (closer to 1 = narrower notch), `mu`
    /// adaptation step, `omega0` initial normalized frequency (cycles/sample).
    pub fn new(rho: f64, mu: f64, omega0: f64) -> Self {
        assert!(rho > 0.0 && rho < 1.0, "rho must be in (0,1)");
        assert!(mu > 0.0, "mu must be > 0");
        Self {
            rho,
            mu,
            omega: (omega0 * 2.0 * std::f64::consts::PI).clamp(1e-4, std::f64::consts::PI - 1e-4),
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            d1: 0.0,
            d2: 0.0,
        }
    }
    pub fn update(&mut self, x: f64) -> FrequencyEstimate {
        let cosw = self.omega.cos();
        let sinw = self.omega.sin();
        let rho2 = self.rho * self.rho;
        // Notch output.
        let y = x - 2.0 * cosw * self.x1 + self.x2 + 2.0 * self.rho * cosw * self.y1
            - rho2 * self.y2;
        // Sensitivity filter g = dy/domega.
        let g = 2.0 * sinw * (self.x1 + self.rho * self.y1)
            + 2.0 * self.rho * cosw * self.d1
            - rho2 * self.d2;
        // Gradient descent on output power y^2.
        self.omega -= self.mu * y * g;
        self.omega = self.omega.clamp(1e-4, std::f64::consts::PI - 1e-4);
        // Shift states.
        self.d2 = self.d1;
        self.d1 = g;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        FrequencyEstimate {
            frequency: self.omega / (2.0 * std::f64::consts::PI),
            filtered: y,
        }
    }
    /// Current estimated normalized frequency (cycles/sample).
    #[inline]
    pub fn frequency(&self) -> f64 {
        self.omega / (2.0 * std::f64::consts::PI)
    }
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
        self.d1 = 0.0;
        self.d2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn savgol_preserves_linear() {
        // A linear ramp should be reproduced exactly by SG (order >= 1).
        let mut sg = SavitzkyGolay::new(5, 2);
        let mut out = None;
        for i in 0..10 {
            out = sg.update(i as f64);
        }
        // Center of last window [5,6,7,8,9] is 7.
        assert!((out.unwrap() - 7.0).abs() < 1e-9);
    }

    #[test]
    fn savgol_derivative_of_linear_is_slope() {
        let mut sg = SavitzkyGolay::with_derivative(5, 2, 1);
        let mut out = None;
        for i in 0..10 {
            out = sg.update(3.0 * i as f64 + 1.0);
        }
        assert!((out.unwrap() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn kz_smooths_constant() {
        let mut kz = KolmogorovZurbenko::new(3, 2);
        let mut out = None;
        for _ in 0..10 {
            out = kz.update(5.0);
        }
        assert!((out.unwrap() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn wiener_constant_signal() {
        let mut w = WienerFilter::new(5, 5);
        let mut out = None;
        for _ in 0..12 {
            out = w.update(2.0);
        }
        assert!((out.unwrap() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn adaptive_notch_locks_onto_tone() {
        // Inject a sinusoid at 0.1 cycles/sample; the notch frequency estimate
        // should converge near 0.1 and the filtered output should shrink.
        let mut anf = AdaptiveNotchFilter::new(0.95, 1e-3, 0.02);
        let f = 0.1;
        let mut freq = 0.0;
        let mut early_power = 0.0;
        let mut late_power = 0.0;
        for i in 0..4000 {
            let x = (2.0 * std::f64::consts::PI * f * i as f64).sin();
            let est = anf.update(x);
            freq = est.frequency;
            if i < 200 {
                early_power += est.filtered * est.filtered;
            }
            if i > 3800 {
                late_power += est.filtered * est.filtered;
            }
        }
        assert!((freq - f).abs() < 0.02, "notch freq {} vs {}", freq, f);
        assert!(late_power < early_power, "notch should suppress the tone");
    }
}
