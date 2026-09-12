//! PyO3 wrappers for the signal-processing feature modules.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::core::traits::ScalarIndicator;
use crate::core::window::WindowKind;
use crate::python::result_types::{
    ChangeResult, FrequencyEstimate, HilbertResult, PredictionResult, SpectralShapeResult,
    SpectrumResult, StateEstimate, WaveletResult,
};
use crate::python::{chk, chk2, chk_pos, chk_slice};
use crate::signal::filters::{
    AdaptiveNotchFilter as CoreAdaptiveNotch, KolmogorovZurbenko as CoreKZ,
    SavitzkyGolay as CoreSG, WienerFilter as CoreWiener,
};
use crate::signal::kalman::{
    AdaptiveKalman as CoreAdaptiveKalman, AlphaBetaTracker as CoreAlphaBeta,
    ConstantVelocityKalman as CoreKalman, ExtendedKalman as CoreEKF, LmsFilter as CoreLms,
    RlsFilter as CoreRls, SquareRootKalman as CoreSqrtKalman, UnscentedKalman as CoreUKF,
};
use crate::signal::prediction::LatticePredictionErrorFilter as CoreLattice;
use crate::signal::prediction::LpcPredictor as CoreLpc;
use crate::signal::regime::{
    PageHinkley as CorePageHinkley, TeagerKaiser as CoreTeagerKaiser,
    ZeroCrossingRate as CoreZeroCrossing,
};
use crate::signal::spectral::{
    self as spectral, ArSpectrum as CoreArSpectrum, BartlettMethod as CoreBartlett,
    BlackmanTukey as CoreBlackmanTukey, Coherence as CoreCoherence,
    CrossSpectrum as CoreCrossSpectrum, FftSpectralDensity as CoreFftSpectral,
    Goertzel as CoreGoertzel, MultitaperPsd as CoreMultitaper, Periodogram as CorePeriodogram,
    WelchPsd as CoreWelch,
};
use crate::signal::timefreq::{
    HilbertTransform as CoreHilbert, ShortTimeFourierTransform as CoreStft,
};
use crate::signal::wavelet::{
    CwtMorlet as CoreCwtMorlet, DiscreteWaveletTransform as CoreDwt, Modwt as CoreModwt,
    MultiresolutionAnalysis as CoreMra, WaveletCoherence as CoreWaveletCoherence,
    WaveletCorrelation as CoreWaveletCorrelation, WaveletPacket as CoreWaveletPacket,
    WaveletVariance as CoreWaveletVariance,
};

fn parse_window(s: &str) -> PyResult<WindowKind> {
    WindowKind::parse(s).ok_or_else(|| PyValueError::new_err(format!("unknown window: {s}")))
}

// ---------------------------------------------------------------------------
// Filters
// ---------------------------------------------------------------------------

/// Savitzky-Golay polynomial smoothing over a rolling window.
///
/// ```text
/// y_t = sum_{k=-m}^{m} c_k x_{t+k}
/// c = (A^T A)^{-1} A^T   (least-squares fit of a polynomial of order p)
/// ```
///
/// Fits a low-order polynomial in a sliding odd window and evaluates it (or
/// its `deriv`-th derivative) at the center.
#[pyclass(module = "qstream")]
pub struct SavitzkyGolay {
    inner: CoreSG,
}

#[pymethods]
impl SavitzkyGolay {
    #[new]
    #[pyo3(signature = (window = 9, order = 2, deriv = 0))]
    fn new(window: usize, order: usize, deriv: usize) -> PyResult<Self> {
        if window % 2 == 0 {
            return Err(PyValueError::new_err("window must be odd"));
        }
        if window <= order {
            return Err(PyValueError::new_err("window must exceed order"));
        }
        let inner = if deriv == 0 {
            CoreSG::new(window, order)
        } else {
            CoreSG::with_derivative(window, order, deriv)
        };
        Ok(Self { inner })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(ScalarIndicator::update(&mut self.inner, value))
    }
    /// Batch update: process many values in a single Python->Rust
    /// crossing. Returns one output per input (`None` while warming up).
    fn update_many(&mut self, values: Vec<f64>) -> PyResult<Vec<Option<f64>>> {
        chk_slice(&values)?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            out.push(self.update(v)?);
        }
        Ok(out)
    }
    fn reset(&mut self) {
        ScalarIndicator::reset(&mut self.inner);
    }
    fn __repr__(&self) -> String {
        "SavitzkyGolay()".to_string()
    }
}

/// Kolmogorov-Zurbenko filter (cascaded moving averages).
///
/// ```text
/// y_t = MA_w^p (x_t)     # p passes of a w-point moving average
/// ```
///
/// Repeated moving-average smoothing removes high-frequency noise while
/// preserving the low-frequency trend with a sharp transition band.
#[pyclass(module = "qstream")]
pub struct KolmogorovZurbenko {
    inner: CoreKZ,
}

#[pymethods]
impl KolmogorovZurbenko {
    #[new]
    #[pyo3(signature = (window = 11, passes = 3))]
    fn new(window: usize, passes: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(passes, "passes")?;
        Ok(Self { inner: CoreKZ::new(window, passes) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(ScalarIndicator::update(&mut self.inner, value))
    }
    /// Batch update: process many values in a single Python->Rust
    /// crossing. Returns one output per input (`None` while warming up).
    fn update_many(&mut self, values: Vec<f64>) -> PyResult<Vec<Option<f64>>> {
        chk_slice(&values)?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            out.push(self.update(v)?);
        }
        Ok(out)
    }
    fn reset(&mut self) {
        ScalarIndicator::reset(&mut self.inner);
    }
    fn __repr__(&self) -> String {
        "KolmogorovZurbenko()".to_string()
    }
}

/// Local (Wiener2-style) adaptive denoiser.
///
/// ```text
/// y = mu + max(0, sigma^2 - nu^2) / sigma^2 * (x - mu)
/// ```
///
/// Per-pixel (per-sample) Wiener filtering: estimates local mean `mu` and
/// variance `sigma^2`, and shrinks toward the mean by the estimated
/// noise variance `nu^2`.
#[pyclass(module = "qstream")]
pub struct WienerFilter {
    inner: CoreWiener,
}

#[pymethods]
impl WienerFilter {
    #[new]
    #[pyo3(signature = (window = 9, noise_window = 32))]
    fn new(window: usize, noise_window: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(noise_window, "noise_window")?;
        Ok(Self { inner: CoreWiener::new(window, noise_window) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(ScalarIndicator::update(&mut self.inner, value))
    }
    /// Batch update: process many values in a single Python->Rust
    /// crossing. Returns one output per input (`None` while warming up).
    fn update_many(&mut self, values: Vec<f64>) -> PyResult<Vec<Option<f64>>> {
        chk_slice(&values)?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            out.push(self.update(v)?);
        }
        Ok(out)
    }
    fn reset(&mut self) {
        ScalarIndicator::reset(&mut self.inner);
    }
    fn __repr__(&self) -> String {
        "WienerFilter()".to_string()
    }
}

/// Adaptive notch filter frequency estimator. Locks onto and suppresses a
/// dominant sinusoid, returning the estimated normalized frequency (cycles per
/// sample) and the notched output each tick.
///
/// ```text
/// H(z) = (1 - 2 rho cos(w0) z^{-1} + rho^2 z^{-2})
///        / (1 - 2 rho cos(w0_hat) z^{-1} + rho^2 z^{-2})
/// w0_hat <- w0_hat - mu * dE/dw0_hat      (gradient descent)
/// ```
///
/// A second-order notch with a gradient-adapted center frequency `w0_hat`.
#[pyclass(module = "qstream")]
pub struct AdaptiveNotchFilter {
    inner: CoreAdaptiveNotch,
}

#[pymethods]
impl AdaptiveNotchFilter {
    #[new]
    #[pyo3(signature = (rho = 0.95, mu = 1e-3, freq0 = 0.05))]
    fn new(rho: f64, mu: f64, freq0: f64) -> PyResult<Self> {
        chk2(rho, mu)?;
        chk(freq0)?;
        if !(rho > 0.0 && rho < 1.0) {
            return Err(PyValueError::new_err("rho must be in (0,1)"));
        }
        if mu <= 0.0 {
            return Err(PyValueError::new_err("mu must be > 0"));
        }
        Ok(Self { inner: CoreAdaptiveNotch::new(rho, mu, freq0) })
    }
    fn update(&mut self, value: f64) -> PyResult<FrequencyEstimate> {
        chk(value)?;
        let est = self.inner.update(value);
        Ok(FrequencyEstimate { frequency: est.frequency, filtered: Some(est.filtered) })
    }
    #[getter]
    fn frequency(&self) -> f64 {
        self.inner.frequency()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "AdaptiveNotchFilter()".to_string()
    }
}

// ---------------------------------------------------------------------------
// Kalman / state-space / adaptive filters
// ---------------------------------------------------------------------------

/// Alpha-beta (g-h) tracker.
///
/// ```text
/// x_hat <- x_hat + alpha * (z - x_hat)
/// v_hat <- v_hat + (beta / dt) * (z - x_hat)
/// x_pred = x_hat + v_hat * dt
/// ```
///
/// Constant-gain state estimator; a fixed-coefficient special case of the
/// Kalman filter.
#[pyclass(module = "qstream")]
pub struct AlphaBetaTracker {
    inner: CoreAlphaBeta,
}

#[pymethods]
impl AlphaBetaTracker {
    #[new]
    #[pyo3(signature = (alpha = 0.5, beta = 0.1, dt = 1.0))]
    fn new(alpha: f64, beta: f64, dt: f64) -> PyResult<Self> {
        chk2(alpha, beta)?;
        chk(dt)?;
        if !(alpha > 0.0 && alpha <= 1.0) || !(beta >= 0.0 && beta <= 1.0) || dt <= 0.0 {
            return Err(PyValueError::new_err("alpha in (0,1], beta in [0,1], dt > 0"));
        }
        Ok(Self { inner: CoreAlphaBeta::new(alpha, beta, dt) })
    }
    fn update(&mut self, measurement: f64) -> PyResult<StateEstimate> {
        chk(measurement)?;
        Ok(self.inner.update(measurement).into())
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "AlphaBetaTracker()".to_string()
    }
}

/// Linear Kalman filter (constant-velocity model) tracking a scalar series.
///
/// ```text
/// x_pred = F x_est
/// P_pred = F P_est F^T + Q
/// K = P_pred H^T (H P_pred H^T + R)^{-1}
/// x_est = x_pred + K (z - H x_pred)
/// P_est = (I - K H) P_pred
/// ```
///
/// Optimal linear state estimator for a constant-velocity (position +
/// velocity) motion model.
#[pyclass(module = "qstream")]
pub struct KalmanFilter {
    inner: CoreKalman,
}

#[pymethods]
impl KalmanFilter {
    #[new]
    #[pyo3(signature = (dt = 1.0, q = 1e-3, r = 1.0, p0 = 1.0))]
    fn new(dt: f64, q: f64, r: f64, p0: f64) -> PyResult<Self> {
        chk2(dt, q)?;
        chk2(r, p0)?;
        if dt <= 0.0 || q < 0.0 || r <= 0.0 || p0 <= 0.0 {
            return Err(PyValueError::new_err("require dt>0, q>=0, r>0, p0>0"));
        }
        Ok(Self { inner: CoreKalman::new(dt, q, r, p0) })
    }
    fn update(&mut self, measurement: f64) -> PyResult<StateEstimate> {
        chk(measurement)?;
        Ok(self.inner.update(measurement).into())
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "KalmanFilter()".to_string()
    }
}

/// Adaptive 1-D Kalman filter with online measurement-noise estimation.
///
/// ```text
/// R_hat <- R_hat + adapt * (v^2 - P_pred - R_hat)
/// ```
///
/// Standard Kalman update where the measurement-noise variance `R` is
/// re-estimated online from the innovation `v = z - H x_pred`.
#[pyclass(module = "qstream")]
pub struct AdaptiveKalman {
    inner: CoreAdaptiveKalman,
}

#[pymethods]
impl AdaptiveKalman {
    #[new]
    #[pyo3(signature = (q = 1e-3, r = 1e-2, adapt = 0.05))]
    fn new(q: f64, r: f64, adapt: f64) -> PyResult<Self> {
        chk2(q, r)?;
        chk(adapt)?;
        if q < 0.0 || r <= 0.0 || !(adapt > 0.0 && adapt <= 1.0) {
            return Err(PyValueError::new_err("require q>=0, r>0, adapt in (0,1]"));
        }
        Ok(Self { inner: CoreAdaptiveKalman::new(q, r, adapt) })
    }
    fn update(&mut self, measurement: f64) -> PyResult<StateEstimate> {
        chk(measurement)?;
        Ok(self.inner.update(measurement).into())
    }
    #[getter]
    fn measurement_noise(&self) -> f64 {
        self.inner.measurement_noise()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "AdaptiveKalman()".to_string()
    }
}

/// Scalar square-root (covariance-factor) Kalman filter.
///
/// ```text
/// P = S * S^T     (covariance factor)
/// S_est = sqrt((1 - K H) S_pred^2 + K^2 R)
/// ```
///
/// Propagates the square-root of the covariance instead of the covariance
/// itself for improved numerical stability.
#[pyclass(module = "qstream")]
pub struct SquareRootKalman {
    inner: CoreSqrtKalman,
}

#[pymethods]
impl SquareRootKalman {
    #[new]
    #[pyo3(signature = (q = 1e-3, r = 1e-2, s0 = 1.0))]
    fn new(q: f64, r: f64, s0: f64) -> PyResult<Self> {
        chk2(q, r)?;
        chk(s0)?;
        if q < 0.0 || r <= 0.0 || s0 <= 0.0 {
            return Err(PyValueError::new_err("require q>=0, r>0, s0>0"));
        }
        Ok(Self { inner: CoreSqrtKalman::new(q, r, s0) })
    }
    fn update(&mut self, measurement: f64) -> PyResult<StateEstimate> {
        chk(measurement)?;
        Ok(self.inner.update(measurement).into())
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "SquareRootKalman()".to_string()
    }
}

/// Unscented Kalman filter (constant-velocity model).
///
/// ```text
/// X_i = mean +/- sqrt((n+kappa) P)   (sigma points)
/// Y_i = f(X_i)
/// mean = sum w_i Y_i ; P = sum w_i (Y_i - mean)(Y_i - mean)^T + Q
/// ```
///
/// Deterministic sampling (unscented transform) of a nonlinear state model;
/// accurate to second order for the propagated mean/covariance.
#[pyclass(module = "qstream")]
pub struct UnscentedKalman {
    inner: CoreUKF,
}

#[pymethods]
impl UnscentedKalman {
    #[new]
    #[pyo3(signature = (dt = 1.0, q = 1e-3, r = 1.0, p0 = 1.0, nonlinear = false))]
    fn new(dt: f64, q: f64, r: f64, p0: f64, nonlinear: bool) -> PyResult<Self> {
        chk2(dt, q)?;
        chk2(r, p0)?;
        let mut inner = CoreUKF::new(dt, q, r, p0);
        if nonlinear {
            inner = inner.with_nonlinear_measurement();
        }
        Ok(Self { inner })
    }
    fn update(&mut self, measurement: f64) -> PyResult<StateEstimate> {
        chk(measurement)?;
        Ok(self.inner.update(measurement).into())
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "UnscentedKalman()".to_string()
    }
}

/// Extended Kalman filter (constant-velocity model, analytic Jacobian).
///
/// ```text
/// x_pred = f(x_est)
/// F = df/dx |_{x_est}     (Jacobian)
/// P_pred = F P_est F^T + Q
/// K = P_pred H^T (H P_pred H^T + R)^{-1}
/// x_est = x_pred + K (z - h(x_pred))
/// ```
///
/// Kalman filter linearized around the current estimate; handles mildly
/// nonlinear state/measurement models.
#[pyclass(module = "qstream")]
pub struct ExtendedKalman {
    inner: CoreEKF,
}

#[pymethods]
impl ExtendedKalman {
    #[new]
    #[pyo3(signature = (dt = 1.0, q = 1e-3, r = 1.0, p0 = 1.0, nonlinear = false))]
    fn new(dt: f64, q: f64, r: f64, p0: f64, nonlinear: bool) -> PyResult<Self> {
        chk2(dt, q)?;
        chk2(r, p0)?;
        let mut inner = CoreEKF::new(dt, q, r, p0);
        if nonlinear {
            inner = inner.with_nonlinear_measurement();
        }
        Ok(Self { inner })
    }
    fn update(&mut self, measurement: f64) -> PyResult<StateEstimate> {
        chk(measurement)?;
        Ok(self.inner.update(measurement).into())
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "ExtendedKalman()".to_string()
    }
}

/// LMS adaptive one-step predictor.
///
/// ```text
/// y_t = w^T x
/// e_t = x_t - y_t
/// w <- w + 2 mu e_t x
/// ```
///
/// Least-mean-squares adaptive FIR filter; cheap stochastic gradient descent
/// on the mean-squared prediction error.
#[pyclass(module = "qstream")]
pub struct LmsFilter {
    inner: CoreLms,
}

#[pymethods]
impl LmsFilter {
    #[new]
    #[pyo3(signature = (order = 4, mu = 0.01))]
    fn new(order: usize, mu: f64) -> PyResult<Self> {
        chk_pos(order, "order")?;
        chk(mu)?;
        if mu <= 0.0 {
            return Err(PyValueError::new_err("mu must be > 0"));
        }
        Ok(Self { inner: CoreLms::new(order, mu) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(ScalarIndicator::update(&mut self.inner, value))
    }
    /// Batch update: process many values in a single Python->Rust
    /// crossing. Returns one output per input (`None` while warming up).
    fn update_many(&mut self, values: Vec<f64>) -> PyResult<Vec<Option<f64>>> {
        chk_slice(&values)?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            out.push(self.update(v)?);
        }
        Ok(out)
    }
    fn reset(&mut self) {
        ScalarIndicator::reset(&mut self.inner);
    }
    fn __repr__(&self) -> String {
        "LmsFilter()".to_string()
    }
}

/// RLS adaptive one-step predictor.
///
/// ```text
/// K = P x / (lambda + x^T P x)
/// e = x_t - w^T x
/// w <- w + K e
/// P <- (P - K x^T P) / lambda
/// ```
///
/// Recursive least squares: exact (not stochastic) minimization of the
/// exponentially-weighted squared prediction error with forgetting factor
/// `lambda`.
#[pyclass(module = "qstream")]
pub struct RlsFilter {
    inner: CoreRls,
}

#[pymethods]
impl RlsFilter {
    #[new]
    #[pyo3(signature = (order = 4, lam = 1.0))]
    fn new(order: usize, lam: f64) -> PyResult<Self> {
        chk_pos(order, "order")?;
        chk(lam)?;
        if !(lam > 0.0 && lam <= 1.0) {
            return Err(PyValueError::new_err("lam must be in (0,1]"));
        }
        Ok(Self { inner: CoreRls::new(order, lam) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(ScalarIndicator::update(&mut self.inner, value))
    }
    /// Batch update: process many values in a single Python->Rust
    /// crossing. Returns one output per input (`None` while warming up).
    fn update_many(&mut self, values: Vec<f64>) -> PyResult<Vec<Option<f64>>> {
        chk_slice(&values)?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            out.push(self.update(v)?);
        }
        Ok(out)
    }
    #[getter]
    fn coefficients(&self) -> Vec<f64> {
        self.inner.coefficients().to_vec()
    }
    fn reset(&mut self) {
        ScalarIndicator::reset(&mut self.inner);
    }
    fn __repr__(&self) -> String {
        "RlsFilter()".to_string()
    }
}

// ---------------------------------------------------------------------------
// Regime / energy
// ---------------------------------------------------------------------------

/// Page-Hinkley change detector.
///
/// ```text
/// S_t = S_{t-1} + (x_t - mu_t - delta)
/// M_t = min_{s<=t} S_s
/// changed = (S_t - M_t) > threshold
/// ```
///
/// Cumulative-sum change detector for a drift in the mean of a stream.
#[pyclass(module = "qstream")]
pub struct PageHinkley {
    inner: CorePageHinkley,
}

#[pymethods]
impl PageHinkley {
    #[new]
    #[pyo3(signature = (delta = 0.005, threshold = 1.0))]
    fn new(delta: f64, threshold: f64) -> PyResult<Self> {
        chk2(delta, threshold)?;
        Ok(Self { inner: CorePageHinkley::new(delta, threshold) })
    }
    fn update(&mut self, value: f64) -> PyResult<ChangeResult> {
        chk(value)?;
        Ok(self.inner.update(value).into())
    }
    #[getter]
    fn statistic(&self) -> f64 {
        self.inner.statistic()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "PageHinkley()".to_string()
    }
}

/// Teager-Kaiser energy operator (one-step delayed).
///
/// ```text
/// Psi[x_t] = x_{t-1}^2 - x_t x_{t-2}
/// ```
///
/// Discrete energy operator; for a sinusoid of amplitude A and frequency f it
/// returns roughly A^2 sin^2(2 pi f).
#[pyclass(module = "qstream")]
pub struct TeagerKaiser {
    inner: CoreTeagerKaiser,
}

#[pymethods]
impl TeagerKaiser {
    #[new]
    fn new() -> Self {
        Self { inner: CoreTeagerKaiser::new() }
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(ScalarIndicator::update(&mut self.inner, value))
    }
    /// Batch update: process many values in a single Python->Rust
    /// crossing. Returns one output per input (`None` while warming up).
    fn update_many(&mut self, values: Vec<f64>) -> PyResult<Vec<Option<f64>>> {
        chk_slice(&values)?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            out.push(self.update(v)?);
        }
        Ok(out)
    }
    fn reset(&mut self) {
        ScalarIndicator::reset(&mut self.inner);
    }
    fn __repr__(&self) -> String {
        "TeagerKaiser()".to_string()
    }
}

/// Zero-crossing rate over a rolling window.
///
/// ```text
/// zcr = (1 / N) * sum_t 1[ x_{t-1} x_t < 0 or |x_t| < threshold ]
/// ```
///
/// Fraction of samples that cross (or lie within a threshold of) zero; a
/// coarse estimate of the dominant frequency.
#[pyclass(module = "qstream")]
pub struct ZeroCrossingRate {
    inner: CoreZeroCrossing,
}

#[pymethods]
impl ZeroCrossingRate {
    #[new]
    #[pyo3(signature = (window = 32, threshold = 0.0))]
    fn new(window: usize, threshold: f64) -> PyResult<Self> {
        if window < 2 {
            return Err(PyValueError::new_err("window must be >= 2"));
        }
        chk(threshold)?;
        Ok(Self { inner: CoreZeroCrossing::new(window, threshold) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(self.inner.update(value))
    }
    /// Batch update: process many values in a single Python->Rust
    /// crossing. Returns one output per input (`None` while warming up).
    fn update_many(&mut self, values: Vec<f64>) -> PyResult<Vec<Option<f64>>> {
        chk_slice(&values)?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            out.push(self.update(v)?);
        }
        Ok(out)
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "ZeroCrossingRate()".to_string()
    }
}

// ---------------------------------------------------------------------------
// Spectral
// ---------------------------------------------------------------------------

macro_rules! spectral_indicator {
    ($name:ident, $core:ty, $doc:literal) => {
        #[doc = $doc]
        #[pyclass(module = "qstream")]
        pub struct $name {
            inner: $core,
        }
        #[pymethods]
        impl $name {
            #[new]
            #[pyo3(signature = (window = 256, update_every = 16, fs = 1.0, window_type = "hann"))]
            fn new(
                window: usize,
                update_every: usize,
                fs: f64,
                window_type: &str,
            ) -> PyResult<Self> {
                chk_pos(window, "window")?;
                chk_pos(update_every, "update_every")?;
                chk(fs)?;
                let kind = parse_window(window_type)?;
                Ok(Self {
                    inner: <$core>::with_options(window, update_every, fs, kind),
                })
            }
            fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
                chk(value)?;
                Ok(self.inner.update(value).map(Into::into))
            }
            fn reset(&mut self) {
                self.inner.reset();
            }
            fn __repr__(&self) -> String {
                stringify!($name).to_string()
            }
        }
    };
}

spectral_indicator!(WelchPsd, CoreWelch, "Welch overlapped-periodogram PSD.\n\n```text\nP(f) = (1/K) * sum_k |X_k(f)|^2 / (fs * sum w^2)\n```\n\nAverages K windowed (typically Hann) segments with overlap to reduce variance; `update_every` controls the recompute cadence.");
spectral_indicator!(Periodogram, CorePeriodogram, "Single-window periodogram.\n\n```text\nP(f) = |X(f)|^2 / (fs * sum w^2)\n```\n\nRaw power estimate of one window; high variance but unbiased.");
spectral_indicator!(
    FftSpectralDensity,
    CoreFftSpectral,
    "FFT spectral density (windowed periodogram).\n\n```text\nS(f) = 2 * |X(f)|^2 / (fs * sum w^2)\n```\n\nOne-sided density with a user-selectable taper (default Hann)."
);
spectral_indicator!(
    BartlettMethod,
    CoreBartlett,
    "Bartlett averaged-segment periodogram.\n\n```text\nP(f) = (1/K) * sum_k |X_k(f)|^2 / (fs * sum w^2)\n```\n\nAverages non-overlapping segments, reducing variance at the cost of resolution."
);

/// Goertzel DFT at a set of target normalized frequencies.
///
/// ```text
/// s_k = x_t + 2 cos(2 pi f_k) s_{k-1} - s_{k-2}
/// |X(f_k)|^2 = s_k^2 + s_{k-1}^2 - 2 cos(2 pi f_k) s_k s_{k-1}
/// ```
///
/// Efficient single-bin DFT recursion; ideal for detecting a few known tones.
#[pyclass(module = "qstream")]
pub struct Goertzel {
    inner: CoreGoertzel,
}

#[pymethods]
impl Goertzel {
    #[new]
    #[pyo3(signature = (frequencies, block = 64))]
    fn new(frequencies: Vec<f64>, block: usize) -> PyResult<Self> {
        chk_slice(&frequencies)?;
        if frequencies.is_empty() {
            return Err(PyValueError::new_err("need at least one frequency"));
        }
        if block < 2 {
            return Err(PyValueError::new_err("block must be >= 2"));
        }
        Ok(Self { inner: CoreGoertzel::new(frequencies, block) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "Goertzel()".to_string()
    }
}

/// Autoregressive (Burg) spectral density.
///
/// ```text
/// P(f) = sigma^2 / |1 + sum_{i=1}^{p} a_i e^{-j 2 pi f i}|^2
/// ```
///
/// AR(p) spectrum from Burg reflection coefficients and the prediction-error
/// variance `sigma^2`.
#[pyclass(module = "qstream")]
pub struct ArSpectrum {
    inner: CoreArSpectrum,
}

#[pymethods]
impl ArSpectrum {
    #[new]
    #[pyo3(signature = (window = 128, order = 16, nfft = 128, update_every = 32))]
    fn new(window: usize, order: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(order, "order")?;
        if order + 1 >= window {
            return Err(PyValueError::new_err("order must be < window-1"));
        }
        Ok(Self { inner: CoreArSpectrum::new(window, order, nfft, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "ArSpectrum()".to_string()
    }
}

/// Blackman-Tukey spectral estimation (FFT of lag-windowed autocorrelation).
///
/// ```text
/// P(f) = sum_{h=-M}^{M} w_h r_h e^{-j 2 pi f h}
/// ```
///
/// Fourier transform of the lag-windowed autocorrelation `r_h` with a
/// triangular (Bartlett) lag window `w_h`.
#[pyclass(module = "qstream")]
pub struct BlackmanTukey {
    inner: CoreBlackmanTukey,
}

#[pymethods]
impl BlackmanTukey {
    #[new]
    #[pyo3(signature = (window = 128, max_lag = 32, update_every = 32))]
    fn new(window: usize, max_lag: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(max_lag, "max_lag")?;
        Ok(Self { inner: CoreBlackmanTukey::new(window, max_lag, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "BlackmanTukey()".to_string()
    }
}

/// Thomson multitaper PSD (DPSS tapers).
///
/// ```text
/// P(f) = (1/K) * sum_{k=1}^{K} |sum_t v_k[t] x_t e^{-j 2 pi f t}|^2
/// ```
///
/// Averages K eigenspectra computed with orthogonal Slepian (DPSS) tapers
/// `v_k`, minimizing leakage and variance.
#[pyclass(module = "qstream")]
pub struct MultitaperPsd {
    inner: CoreMultitaper,
}

#[pymethods]
impl MultitaperPsd {
    #[new]
    #[pyo3(signature = (window = 256, nw = 3.0, tapers = 5, update_every = 32))]
    fn new(window: usize, nw: f64, tapers: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(tapers, "tapers")?;
        chk(nw)?;
        Ok(Self { inner: CoreMultitaper::new(window, nw, tapers, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "MultitaperPsd()".to_string()
    }
}

macro_rules! cross_spectral_indicator {
    ($name:ident, $core:ty, $doc:literal) => {
        #[doc = $doc]
        #[pyclass(module = "qstream")]
        pub struct $name {
            inner: $core,
        }
        #[pymethods]
        impl $name {
            #[new]
            #[pyo3(signature = (window = 256, update_every = 16, fs = 1.0, window_type = "hann"))]
            fn new(
                window: usize,
                update_every: usize,
                fs: f64,
                window_type: &str,
            ) -> PyResult<Self> {
                chk_pos(window, "window")?;
                chk_pos(update_every, "update_every")?;
                chk(fs)?;
                let kind = parse_window(window_type)?;
                Ok(Self { inner: <$core>::with_options(window, update_every, fs, kind) })
            }
            fn update(&mut self, x: f64, y: f64) -> PyResult<Option<SpectrumResult>> {
                chk2(x, y)?;
                Ok(self.inner.update(x, y).map(Into::into))
            }
            fn reset(&mut self) {
                self.inner.reset();
            }
            fn __repr__(&self) -> String {
                stringify!($name).to_string()
            }
        }
    };
}

cross_spectral_indicator!(CrossSpectrum, CoreCrossSpectrum, "Cross-spectral density magnitude.\n\n```text\n|S_xy(f)| = |E[X(f) Y*(f)]| / (fs * sum w^2)\n```\n\nWelch-averaged cross-spectrum between two streams.");
cross_spectral_indicator!(Coherence, CoreCoherence, "Magnitude-squared coherence.\n\n```text\ngamma^2(f) = |S_xy(f)|^2 / (S_xx(f) * S_yy(f))\n```\n\nValues in [0, 1]; 1 means perfect linear relationship at frequency f.");

/// Compute spectral-shape features from a [`SpectrumResult`].
///
/// ```text
/// centroid = sum f P(f) / sum P(f)
/// entropy  = -sum p log p / log N
/// flatness = exp(mean log P) / mean P
/// rolloff  = f where cumulative energy reaches 85%
/// ```
#[pyfunction]
pub fn spectral_shape(spectrum: &SpectrumResult) -> SpectralShapeResult {
    let core = spectral::SpectrumResult {
        frequencies: spectrum.frequencies.clone(),
        power: spectrum.power.clone(),
        dominant_frequency: spectrum.dominant_frequency,
        peak_power: spectrum.peak_power,
    };
    spectral::spectral_shape(&core).into()
}

// ---------------------------------------------------------------------------
// Time-frequency (STFT / Hilbert)
// ---------------------------------------------------------------------------

/// Short-Time Fourier Transform producing a running spectrogram.
///
/// ```text
/// X(t, f) = sum_k w[k] x_{t+k} e^{-j 2 pi f k}
/// ```
///
/// Each hop returns the magnitude spectrum of the current windowed frame;
/// `.spectrogram()` returns the accumulated time-frequency matrix
/// `(frames, bins, flat_values)`.
#[pyclass(module = "qstream")]
pub struct ShortTimeFourierTransform {
    inner: CoreStft,
}

#[pymethods]
impl ShortTimeFourierTransform {
    #[new]
    #[pyo3(signature = (window = 256, hop = 128, fs = 1.0, window_type = "hann", max_frames = 64))]
    fn new(
        window: usize,
        hop: usize,
        fs: f64,
        window_type: &str,
        max_frames: usize,
    ) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(hop, "hop")?;
        chk(fs)?;
        let kind = parse_window(window_type)?;
        Ok(Self {
            inner: CoreStft::new(window, hop, fs, kind, max_frames),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    /// Accumulated spectrogram as `(n_frames, n_bins, flat_magnitudes)`.
    fn spectrogram(&self) -> (usize, usize, Vec<f64>) {
        self.inner.spectrogram()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "ShortTimeFourierTransform()".to_string()
    }
}

/// Hilbert transform (analytic signal).
///
/// ```text
/// z_t = x_t + j H[x_t]
/// amplitude = |z_t|
/// phase = arg(z_t)
/// frequency = d(phase)/dt
/// ```
///
/// On each cadence returns the instantaneous amplitude (envelope), phase, and
/// frequency at the window center.
#[pyclass(module = "qstream")]
pub struct HilbertTransform {
    inner: CoreHilbert,
}

#[pymethods]
impl HilbertTransform {
    #[new]
    #[pyo3(signature = (window = 128, update_every = 64, fs = 1.0))]
    fn new(window: usize, update_every: usize, fs: f64) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(update_every, "update_every")?;
        chk(fs)?;
        Ok(Self { inner: CoreHilbert::new(window, update_every, fs) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<HilbertResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(|(amplitude, phase, frequency)| HilbertResult {
            amplitude,
            phase,
            frequency: Some(frequency),
        }))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "HilbertTransform()".to_string()
    }
}

/// Instantaneous frequency via the Hilbert analytic signal.
///
/// ```text
/// f_t = (1 / 2 pi) * d(arg(x_t + j H[x_t])) / dt
/// ```
///
/// Convenience alias over [`HilbertTransform`]; returns the frequency at the
/// window center.
#[pyclass(module = "qstream")]
pub struct InstantaneousFrequency {
    inner: CoreHilbert,
}

#[pymethods]
impl InstantaneousFrequency {
    #[new]
    #[pyo3(signature = (window = 128, update_every = 64, fs = 1.0))]
    fn new(window: usize, update_every: usize, fs: f64) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(update_every, "update_every")?;
        chk(fs)?;
        Ok(Self { inner: CoreHilbert::new(window, update_every, fs) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(self.inner.update(value).map(|(_a, _p, f)| f))
    }
    /// Batch update: process many values in a single Python->Rust
    /// crossing. Returns one output per input (`None` while warming up).
    fn update_many(&mut self, values: Vec<f64>) -> PyResult<Vec<Option<f64>>> {
        chk_slice(&values)?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            out.push(self.update(v)?);
        }
        Ok(out)
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "InstantaneousFrequency()".to_string()
    }
}

// ---------------------------------------------------------------------------
// Wavelets
// ---------------------------------------------------------------------------

macro_rules! wavelet_indicator {
    ($name:ident, $core:ty, $levels:expr, $doc:literal) => {
        #[doc = $doc]
        #[pyclass(module = "qstream")]
        pub struct $name {
            inner: $core,
        }
        #[pymethods]
        impl $name {
            #[new]
            #[pyo3(signature = (window = 64, levels = $levels, update_every = 16))]
            fn new(window: usize, levels: usize, update_every: usize) -> PyResult<Self> {
                chk_pos(window, "window")?;
                chk_pos(levels, "levels")?;
                chk_pos(update_every, "update_every")?;
                Ok(Self { inner: <$core>::new(window, levels, update_every) })
            }
            fn update(&mut self, value: f64) -> PyResult<Option<WaveletResult>> {
                chk(value)?;
                Ok(self.inner.update(value).map(Into::into))
            }
            fn reset(&mut self) {
                self.inner.reset();
            }
            fn __repr__(&self) -> String {
                stringify!($name).to_string()
            }
        }
    };
}

wavelet_indicator!(Modwt, CoreModwt, 3, "MODWT detail coefficients per scale.\n\n```text\nW_j,t = sum_k h_j[k] * x_{t-k}\n```\n\nUndecimated (shift-invariant) wavelet detail coefficients at dyadic scales j.");
wavelet_indicator!(
    MultiresolutionAnalysis,
    CoreMra,
    3,
    "Wavelet multiresolution analysis per scale.\n\n```text\nx_t = A_J,t + sum_{j<=J} D_j,t\n```\n\nDecomposes the signal into approximation (A) and detail (D) components at dyadic scales."
);
wavelet_indicator!(
    WaveletVariance,
    CoreWaveletVariance,
    3,
    "Wavelet variance per scale (MODWT).\n\n```text\nsigma^2_j = Var(W_j)\n```\n\nVariance of MODWT coefficients, partitioning total variance across scales."
);
wavelet_indicator!(WaveletPacket, CoreWaveletPacket, 2, "Haar wavelet packet decomposition.\n\n```text\nW_j,b = sum_k h[k] * x_{t-k}\n```\n\nFull binary tree of wavelet coefficients (approximation + detail at every node).");

/// Haar discrete wavelet transform over a rolling power-of-two window.
///
/// ```text
/// a_j[k] = (a_{j-1}[2k] + a_{j-1}[2k+1]) / sqrt(2)
/// d_j[k] = (a_{j-1}[2k] - a_{j-1}[2k+1]) / sqrt(2)
/// ```
///
/// Decimated two-channel filter bank; returns approximation `a` and detail `d`
/// coefficients at dyadic scales.
#[pyclass(module = "qstream")]
pub struct DiscreteWaveletTransform {
    inner: CoreDwt,
}

#[pymethods]
impl DiscreteWaveletTransform {
    #[new]
    #[pyo3(signature = (window = 64, update_every = 16))]
    fn new(window: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(update_every, "update_every")?;
        Ok(Self { inner: CoreDwt::new(window, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<WaveletResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "DiscreteWaveletTransform()".to_string()
    }
}

/// Morlet continuous wavelet transform evaluated at the latest sample.
///
/// ```text
/// W(a) = sum_t x_t * (1/sqrt(a)) psi*((t - tau)/a)
/// psi(t) = pi^{-1/4} e^{j w0 t} e^{-t^2/2}
/// ```
///
/// Inner product of the signal with a scaled Morlet mother wavelet at the
/// given dyadic scales `a`.
#[pyclass(module = "qstream")]
pub struct CwtMorlet {
    inner: CoreCwtMorlet,
}

#[pymethods]
impl CwtMorlet {
    #[new]
    #[pyo3(signature = (window = 64, scales = vec![2.0, 4.0, 8.0, 16.0], update_every = 16))]
    fn new(window: usize, scales: Vec<f64>, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_slice(&scales)?;
        if scales.is_empty() {
            return Err(PyValueError::new_err("need at least one scale"));
        }
        Ok(Self { inner: CoreCwtMorlet::new(window, scales, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<WaveletResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "CwtMorlet()".to_string()
    }
}

macro_rules! cross_wavelet_indicator {
    ($name:ident, $core:ty, $doc:literal) => {
        #[doc = $doc]
        #[pyclass(module = "qstream")]
        pub struct $name {
            inner: $core,
        }
        #[pymethods]
        impl $name {
            #[new]
            #[pyo3(signature = (window = 64, levels = 3, update_every = 16))]
            fn new(window: usize, levels: usize, update_every: usize) -> PyResult<Self> {
                chk_pos(window, "window")?;
                chk_pos(levels, "levels")?;
                chk_pos(update_every, "update_every")?;
                Ok(Self { inner: <$core>::new(window, levels, update_every) })
            }
            fn update(&mut self, x: f64, y: f64) -> PyResult<Option<SpectrumResult>> {
                chk2(x, y)?;
                Ok(self.inner.update(x, y).map(Into::into))
            }
            fn reset(&mut self) {
                self.inner.reset();
            }
            fn __repr__(&self) -> String {
                stringify!($name).to_string()
            }
        }
    };
}

cross_wavelet_indicator!(WaveletCorrelation, CoreWaveletCorrelation, "Wavelet correlation per scale.\n\n```text\nrho_j = Cov(W_x_j, W_y_j) / (sigma_x_j * sigma_y_j)\n```\n\nPer-scale correlation between the MODWT coefficients of two streams.");
cross_wavelet_indicator!(WaveletCoherence, CoreWaveletCoherence, "Wavelet coherence per scale.\n\n```text\nC_j = |sum W_x_j * W_y_j*|^2 / (sum |W_x_j|^2 * sum |W_y_j|^2)\n```\n\nPer-scale magnitude-squared coherence in [0, 1].");

// ---------------------------------------------------------------------------
// Prediction
// ---------------------------------------------------------------------------

/// Streaming LPC / Levinson-Durbin predictor.
///
/// ```text
/// x_t = sum_{i=1}^{p} a_i x_{t-i} + e_t
/// prediction = sum_i a_i x_{t-i}
/// ```
///
/// Linear predictive coding: AR(p) coefficients via Levinson-Durbin, with the
/// one-step prediction output every tick.
#[pyclass(module = "qstream")]
pub struct LpcPredictor {
    inner: CoreLpc,
}

#[pymethods]
impl LpcPredictor {
    #[new]
    #[pyo3(signature = (window = 64, order = 8, update_every = 8))]
    fn new(window: usize, order: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(order, "order")?;
        if order + 1 >= window {
            return Err(PyValueError::new_err("order must be < window-1"));
        }
        Ok(Self { inner: CoreLpc::new(window, order, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<PredictionResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    #[getter]
    fn coefficients(&self) -> Vec<f64> {
        self.inner.coefficients().to_vec()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "LpcPredictor()".to_string()
    }
}

/// Lattice prediction-error filter (outputs the whitened residual).
///
/// ```text
/// f_m = f_{m-1} - k_m b_{m-1}
/// b_m = b_{m-1} - k_m f_{m-1}
/// residual = f_p = x_t - sum_i a_i x_{t-i}
/// ```
///
/// Forward/backward lattice recursion driven by Burg reflection coefficients
/// `k_m`; outputs the order-p forward prediction error.
#[pyclass(module = "qstream")]
pub struct LatticePredictionErrorFilter {
    inner: CoreLattice,
}

#[pymethods]
impl LatticePredictionErrorFilter {
    #[new]
    #[pyo3(signature = (window = 64, order = 8, update_every = 8))]
    fn new(window: usize, order: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(order, "order")?;
        if order + 1 >= window {
            return Err(PyValueError::new_err("order must be < window-1"));
        }
        Ok(Self { inner: CoreLattice::new(window, order, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(self.inner.update(value))
    }
    /// Batch update: process many values in a single Python->Rust
    /// crossing. Returns one output per input (`None` while warming up).
    fn update_many(&mut self, values: Vec<f64>) -> PyResult<Vec<Option<f64>>> {
        chk_slice(&values)?;
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            out.push(self.update(v)?);
        }
        Ok(out)
    }
    #[getter]
    fn reflection_coefficients(&self) -> Vec<f64> {
        self.inner.reflection_coefficients().to_vec()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "LatticePredictionErrorFilter()".to_string()
    }
}

/// Levinson-Durbin recursion on an autocorrelation vector (batch helper).
///
/// ```text
/// r_m = r_xx(m) - sum_{i=1}^{m-1} a_i r_xx(m-i)
/// k_m = r_m / e_{m-1}
/// e_m = e_{m-1} (1 - k_m^2)
/// ```
///
/// Returns `(ar_coefficients, error_variance, reflection_coefficients)`.
#[pyfunction]
#[pyo3(signature = (acf, order))]
pub fn levinson_durbin(acf: Vec<f64>, order: usize) -> PyResult<(Vec<f64>, f64, Vec<f64>)> {
    chk_slice(&acf)?;
    chk_pos(order, "order")?;
    if acf.len() < order + 1 {
        return Err(PyValueError::new_err("acf must have at least order+1 elements"));
    }
    let (a, e, k) = crate::signal::prediction::levinson_durbin(&acf, order);
    Ok((a, e, k))
}

/// Burg AR estimation on a data window (batch helper).
///
/// ```text
/// k_m = 2 sum e_f[t] e_b[t-1] / sum (e_f[t]^2 + e_b[t-1]^2)
/// ```
///
/// Returns `(ar_coefficients, reflection_coefficients)` from the forward and
/// backward prediction errors.
#[pyfunction]
#[pyo3(signature = (data, order))]
pub fn burg_ar(data: Vec<f64>, order: usize) -> PyResult<(Vec<f64>, Vec<f64>)> {
    chk_slice(&data)?;
    chk_pos(order, "order")?;
    let (a, k) = crate::signal::prediction::burg_ar(&data, order);
    Ok((a, k))
}
