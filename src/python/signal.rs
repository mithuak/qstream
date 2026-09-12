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

spectral_indicator!(WelchPsd, CoreWelch, "Welch overlapped-periodogram PSD.");
spectral_indicator!(Periodogram, CorePeriodogram, "Single-window periodogram.");
spectral_indicator!(
    FftSpectralDensity,
    CoreFftSpectral,
    "FFT spectral density (windowed periodogram)."
);
spectral_indicator!(
    BartlettMethod,
    CoreBartlett,
    "Bartlett averaged-segment periodogram."
);

/// Goertzel DFT at a set of target normalized frequencies.
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

cross_spectral_indicator!(CrossSpectrum, CoreCrossSpectrum, "Cross-spectral density magnitude.");
cross_spectral_indicator!(Coherence, CoreCoherence, "Magnitude-squared coherence.");

/// Compute spectral-shape features from a [`SpectrumResult`].
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

/// Short-Time Fourier Transform producing a running spectrogram. Each hop
/// returns the magnitude spectrum of the current window; call `.spectrogram()`
/// for the accumulated time-frequency matrix `(frames, bins, flat_values)`.
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

/// Hilbert transform (analytic signal). On each cadence returns the
/// instantaneous amplitude (envelope), phase, and frequency at the window
/// center.
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

/// Instantaneous frequency via the Hilbert analytic signal (convenience alias
/// over [`HilbertTransform`]); returns the frequency at the window center.
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

wavelet_indicator!(Modwt, CoreModwt, 3, "MODWT detail coefficients per scale.");
wavelet_indicator!(
    MultiresolutionAnalysis,
    CoreMra,
    3,
    "Wavelet multiresolution analysis per scale."
);
wavelet_indicator!(
    WaveletVariance,
    CoreWaveletVariance,
    3,
    "Wavelet variance per scale (MODWT)."
);
wavelet_indicator!(WaveletPacket, CoreWaveletPacket, 2, "Haar wavelet packet decomposition.");

/// Haar discrete wavelet transform over a rolling power-of-two window.
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

cross_wavelet_indicator!(WaveletCorrelation, CoreWaveletCorrelation, "Wavelet correlation per scale.");
cross_wavelet_indicator!(WaveletCoherence, CoreWaveletCoherence, "Wavelet coherence per scale.");

// ---------------------------------------------------------------------------
// Prediction
// ---------------------------------------------------------------------------

/// Streaming LPC / Levinson-Durbin predictor.
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
#[pyfunction]
#[pyo3(signature = (data, order))]
pub fn burg_ar(data: Vec<f64>, order: usize) -> PyResult<(Vec<f64>, Vec<f64>)> {
    chk_slice(&data)?;
    chk_pos(order, "order")?;
    let (a, k) = crate::signal::prediction::burg_ar(&data, order);
    Ok((a, k))
}
