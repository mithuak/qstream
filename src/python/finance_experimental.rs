//! PyO3 wrappers for Phase 7 experimental finance features.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::python::{chk, chk_pos, chk_slice};
use crate::finance::portfolio_optimizers::{
    ExponentiallyWeightedPortfolio as CoreEWP, MaxDiversification as CoreMaxDiv,
};
use crate::finance::realized_volatility::{
    RealizedKernel as CoreRK, TwoScaleRealizedVariance as CoreTSRV,
};
use crate::finance::tail_risk::{
    EntropicVaR as CoreEVaR, EVTGpdTailRisk as CoreEVT, EVTResult, JohnsonSUVaR as CoreJSU,
    SpectralRiskMeasure as CoreSRM,
};
use crate::finance::volatility_derivatives::{
    DemeterfiVarianceSwap as CoreDemeterfi, FlemingOstdiekWhaleyVIX as CoreFOW,
    VandermeerVIX as CoreVandermeer, VarianceSwapResult,
};

// ===========================================================================
// EVT Result
// ===========================================================================

#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct PyEVTResult {
    #[pyo3(get)]
    pub var: f64,
    #[pyo3(get)]
    pub expected_shortfall: f64,
    #[pyo3(get)]
    pub shape: f64,
    #[pyo3(get)]
    pub scale: f64,
    #[pyo3(get)]
    pub n_exceedances: usize,
}

impl From<EVTResult> for PyEVTResult {
    fn from(r: EVTResult) -> Self {
        Self {
            var: r.var,
            expected_shortfall: r.expected_shortfall,
            shape: r.shape,
            scale: r.scale,
            n_exceedances: r.n_exceedances,
        }
    }
}

// ===========================================================================
// Variance Swap Result
// ===========================================================================

#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct PyVarianceSwapResult {
    #[pyo3(get)]
    pub strike: f64,
    #[pyo3(get)]
    pub realized_variance: f64,
    #[pyo3(get)]
    pub correction: f64,
}

impl From<VarianceSwapResult> for PyVarianceSwapResult {
    fn from(r: VarianceSwapResult) -> Self {
        Self {
            strike: r.strike,
            realized_variance: r.realized_variance,
            correction: r.correction,
        }
    }
}

// ===========================================================================
// Tail Risk Measures
// ===========================================================================

/// Entropic Value-at-Risk (EVaR).
///
/// ```text
/// EVaR = inf_{z>0} ( ln E[e^{z X}] - ln(alpha) ) / z
/// ```
///
/// Coherent tail-risk measure from the Chernoff bound on the loss
/// distribution.
#[pyclass(module = "qstream")]
pub struct EntropicVaR {
    inner: CoreEVaR,
}

#[pymethods]
impl EntropicVaR {
    #[new]
    #[pyo3(signature = (window = 64, alpha = 0.01, update_every = 16))]
    fn new(window: usize, alpha: f64, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(PyValueError::new_err("alpha must be in (0,1)"));
        }
        Ok(Self {
            inner: CoreEVaR::new(window, alpha, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(self.inner.update(value))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// EVT Peaks-Over-Threshold / GPD tail risk.
///
/// ```text
/// VaR = u + (beta/xi) * ((N/Nu * (1-alpha))^{-xi} - 1)
/// ES  = (VaR + beta - xi u) / (1 - xi)
/// ```
///
/// Fits a Generalized Pareto Distribution to exceedances over a high
/// threshold and computes VaR / expected shortfall.
#[pyclass(module = "qstream")]
pub struct EVTGpdTailRisk {
    inner: CoreEVT,
}

#[pymethods]
impl EVTGpdTailRisk {
    #[new]
    #[pyo3(signature = (window = 128, alpha = 0.01, threshold_quantile = 0.95, update_every = 32))]
    fn new(
        window: usize,
        alpha: f64,
        threshold_quantile: f64,
        update_every: usize,
    ) -> PyResult<Self> {
        chk_pos(window, "window")?;
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(PyValueError::new_err("alpha must be in (0,1)"));
        }
        Ok(Self {
            inner: CoreEVT::new(window, alpha, threshold_quantile, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<PyEVTResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Johnson-SU Value-at-Risk.
///
/// ```text
/// VaR = -(xi + lambda sinh((z_alpha - gamma)/delta))
/// ```
///
/// Fits a Johnson SU distribution (which can match any skewness/kurtosis
/// combination) and reads the `alpha`-quantile.
#[pyclass(module = "qstream")]
pub struct JohnsonSUVaR {
    inner: CoreJSU,
}

#[pymethods]
impl JohnsonSUVaR {
    #[new]
    #[pyo3(signature = (window = 64, alpha = 0.01, update_every = 16))]
    fn new(window: usize, alpha: f64, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(PyValueError::new_err("alpha must be in (0,1)"));
        }
        Ok(Self {
            inner: CoreJSU::new(window, alpha, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(self.inner.update(value))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Spectral Risk Measure (exponential weighting).
///
/// ```text
/// SRM = int_0^1 phi(u) q_u(X) du
/// phi(u) = gamma e^{-gamma(1-u)} / (1 - e^{-gamma})
/// ```
///
/// Weighted average of loss quantiles with exponentially decaying weights
/// controlled by risk aversion `gamma`.
#[pyclass(module = "qstream")]
pub struct SpectralRiskMeasure {
    inner: CoreSRM,
}

#[pymethods]
impl SpectralRiskMeasure {
    #[new]
    #[pyo3(signature = (window = 64, gamma = 0.05, update_every = 16))]
    fn new(window: usize, gamma: f64, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk(gamma)?;
        Ok(Self {
            inner: CoreSRM::new(window, gamma, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(self.inner.update(value))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

// ===========================================================================
// Portfolio Optimizers
// ===========================================================================

/// Maximum Diversification Portfolio.
///
/// ```text
/// w = Sigma^{-1} sigma / (1^T Sigma^{-1} sigma)
/// ```
///
/// Maximizes the diversification ratio `w^T sigma / sqrt(w^T Sigma w)`.
#[pyclass(module = "qstream")]
pub struct MaxDiversification {
    inner: CoreMaxDiv,
}

#[pymethods]
impl MaxDiversification {
    #[new]
    #[pyo3(signature = (n_assets = 3, period = 32, update_every = 8))]
    fn new(n_assets: usize, period: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(n_assets, "n_assets")?;
        chk_pos(period, "period")?;
        Ok(Self {
            inner: CoreMaxDiv::new(n_assets, period, update_every),
        })
    }
    fn update(&mut self, returns: Vec<f64>) -> PyResult<Option<Vec<f64>>> {
        chk_slice(&returns)?;
        Ok(self.inner.update(&returns))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Exponentially Weighted Portfolio (EWP).
///
/// ```text
/// w = Sigma^{-1} 1 / (1^T Sigma^{-1} 1)     (minimum variance)
/// Sigma_t = lambda Sigma_{t-1} + (1-lambda) r r^T
/// ```
///
/// Minimum-variance weights from an exponentially weighted covariance matrix.
#[pyclass(module = "qstream")]
pub struct ExponentiallyWeightedPortfolio {
    inner: CoreEWP,
}

#[pymethods]
impl ExponentiallyWeightedPortfolio {
    #[new]
    #[pyo3(signature = (n_assets = 3, lambda = 0.94))]
    fn new(n_assets: usize, lambda: f64) -> PyResult<Self> {
        chk_pos(n_assets, "n_assets")?;
        if lambda <= 0.0 || lambda >= 1.0 {
            return Err(PyValueError::new_err("lambda must be in (0,1)"));
        }
        Ok(Self {
            inner: CoreEWP::new(n_assets, lambda),
        })
    }
    fn update(&mut self, returns: Vec<f64>) -> PyResult<Option<Vec<f64>>> {
        chk_slice(&returns)?;
        Ok(self.inner.update(&returns))
    }
    fn portfolio_volatility(&self, weights: Vec<f64>) -> f64 {
        self.inner.portfolio_volatility(&weights)
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

// ===========================================================================
// Realized Volatility
// ===========================================================================

/// Realized Kernel with microstructure-noise correction.
///
/// ```text
/// RK = gamma_0 + sum_{h=1}^{H} k(h/(H+1)) (gamma_h + gamma_{-h})
/// ```
///
/// Parzen-kernel weighted realized variance robust to microstructure noise.
#[pyclass(module = "qstream")]
pub struct RealizedKernel {
    inner: CoreRK,
}

#[pymethods]
impl RealizedKernel {
    #[new]
    #[pyo3(signature = (window = 64, n_lags = 10, annualization = 252.0, update_every = 16))]
    fn new(window: usize, n_lags: usize, annualization: f64, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(n_lags, "n_lags")?;
        Ok(Self {
            inner: CoreRK::new(window, n_lags, annualization, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(self.inner.update(value))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Two-Scale Realized Variance (TSRV).
///
/// ```text
/// TSRV = RV_avg - (n/K) RV_sparse
/// ```
///
/// Zhang-Mykland-Ait-Sahalia estimator combining a sparse and an averaged
/// realized variance to cancel microstructure noise.
#[pyclass(module = "qstream")]
pub struct TwoScaleRealizedVariance {
    inner: CoreTSRV,
}

#[pymethods]
impl TwoScaleRealizedVariance {
    #[new]
    #[pyo3(signature = (window = 64, n_subsamples = 5, annualization = 252.0, update_every = 16))]
    fn new(
        window: usize,
        n_subsamples: usize,
        annualization: f64,
        update_every: usize,
    ) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(n_subsamples, "n_subsamples")?;
        Ok(Self {
            inner: CoreTSRV::new(window, n_subsamples, annualization, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(self.inner.update(value))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

// ===========================================================================
// Volatility Derivatives
// ===========================================================================

/// Fleming-Ostdiek-Whaley VIX Implied Volatility.
///
/// ```text
/// sigma^2 = (2/T) sum_K DeltaK/K^2 Q(K) - (1/T)(F/K0 - 1)^2
/// ```
///
/// Model-free VIX-style implied volatility index (simplified to realized
/// variance as a proxy).
#[pyclass(module = "qstream")]
pub struct FlemingOstdiekWhaleyVIX {
    inner: CoreFOW,
}

#[pymethods]
impl FlemingOstdiekWhaleyVIX {
    #[new]
    #[pyo3(signature = (window = 22, annualization = 252.0, update_every = 5))]
    fn new(window: usize, annualization: f64, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreFOW::new(window, annualization, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(self.inner.update(value))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Vandermeer VIX-Implied Volatility Calculator.
///
/// ```text
/// sigma ~ IQR(returns) / 1.35
/// VIX = sigma * sqrt(annualization) * 100
/// ```
///
/// Simplified VIX estimate from the interquartile range of the return
/// distribution.
#[pyclass(module = "qstream")]
pub struct VandermeerVIX {
    inner: CoreVandermeer,
}

#[pymethods]
impl VandermeerVIX {
    #[new]
    #[pyo3(signature = (window = 22, annualization = 252.0, update_every = 5))]
    fn new(window: usize, annualization: f64, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreVandermeer::new(window, annualization, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(self.inner.update(value))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Demeterfi Variance Swap Replication.
///
/// ```text
/// K_var = E[RV] + convexity correction
/// ```
///
/// Fair variance-swap strike from the model-free replication formula.
#[pyclass(module = "qstream")]
pub struct DemeterfiVarianceSwap {
    inner: CoreDemeterfi,
}

#[pymethods]
impl DemeterfiVarianceSwap {
    #[new]
    #[pyo3(signature = (window = 22, period = 0.082, annualization = 252.0, update_every = 5))]
    fn new(
        window: usize,
        period: f64,
        annualization: f64,
        update_every: usize,
    ) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreDemeterfi::new(window, period, annualization, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<PyVarianceSwapResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}
