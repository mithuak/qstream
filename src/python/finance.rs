//! PyO3 wrappers for the finance feature modules. Each class owns its pure-Rust
//! indicator; `update` validates inputs, crosses into Rust, and returns either
//! a scalar/None or a compact result object.

use pyo3::prelude::*;

use crate::core::covariance::EwmaCovariance as CoreEwmaCov;
use crate::core::traits::{HlcIndicator, OhlcIndicator, PairIndicator, ScalarIndicator};
use crate::finance::derivatives::{self as deriv, VarianceSwap as CoreVarianceSwap};
use crate::finance::factor::{
    CapmRegression as CoreCapm, FactorModel as CoreFactorModel, RollingBeta as CoreRollingBeta,
    RollingBetaStability as CoreBetaStability, TrackingError as CoreTrackingError,
    TreynorMazuy as CoreTreynorMazuy,
};
use crate::finance::microstructure::{
    AbdiRanaldo as CoreAbdiRanaldo, CorwinSchultz as CoreCorwinSchultz,
    GlostenMilgrom as CoreGlostenMilgrom,
};
use crate::finance::performance::{
    CaptureRatios as CoreCapture, ConditionalSharpe as CoreCondSharpe,
    DeflatedSharpeRatio as CoreDeflatedSharpe, GainLossRatio as CoreGainLoss,
    LoAutocorrelationSharpe as CoreLoSharpe, OmegaRatio as CoreOmega, SharpeRatio as CoreSharpe,
    SortinoRatio as CoreSortino,
};
use crate::finance::portfolio::{
    ComponentMarginalVaR as CoreComponentVaR, PortfolioDuration as CorePortfolioDuration,
    PortfolioReturns as CorePortfolioReturns, RiskParity as CoreRiskParity,
};
use crate::finance::risk::{
    ConditionalDrawdownAtRisk as CoreCDaR, ConditionalValueAtRisk as CoreCVaR,
    CornishFisherVaR as CoreCornishFisher, RachevRatio as CoreRachev, ValueAtRisk as CoreVaR,
};
use crate::finance::volatility::{
    EwmaVolatility as CoreEwmaVol, EwmaVariance as CoreEwmaVariance, Garch11 as CoreGarch,
    GarmanKlass as CoreGarmanKlass, Parkinson as CoreParkinson,
    RealizedVolatility as CoreRealizedVol, RogersSatchell as CoreRogersSatchell,
    RollingReturns as CoreRollingReturns, RollingVolatility as CoreRollingVol,
};
use crate::python::result_types::{
    make_component_var, CaptureResult, ComponentVaRResult, FactorResult, TermStructureResult,
};
use crate::python::{chk, chk2, chk_pos, chk_slice};

// ---------------------------------------------------------------------------
// Volatility
// ---------------------------------------------------------------------------

/// RiskMetrics-style EWMA volatility. `alpha` is the weight on the new squared
/// return (`lambda = 1 - alpha`); the RiskMetrics daily default is alpha=0.06.
#[pyclass(module = "qstream")]
pub struct EwmaVolatility {
    inner: CoreEwmaVol,
}

#[pymethods]
impl EwmaVolatility {
    #[new]
    #[pyo3(signature = (alpha = 0.06))]
    fn new(alpha: f64) -> PyResult<Self> {
        if !(alpha > 0.0 && alpha < 1.0) {
            return Err(pyo3::exceptions::PyValueError::new_err("alpha must be in (0,1)"));
        }
        Ok(Self { inner: CoreEwmaVol::new(1.0 - alpha) })
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
    fn variance(&self) -> f64 {
        self.inner.variance()
    }
    fn reset(&mut self) {
        ScalarIndicator::reset(&mut self.inner);
    }
    fn __repr__(&self) -> String {
        "EwmaVolatility()".to_string()
    }
}

/// GARCH(1,1) conditional volatility.
#[pyclass(module = "qstream")]
pub struct Garch {
    inner: CoreGarch,
}

#[pymethods]
impl Garch {
    #[new]
    #[pyo3(signature = (omega = 1e-6, alpha = 0.09, beta = 0.90))]
    fn new(omega: f64, alpha: f64, beta: f64) -> PyResult<Self> {
        if omega < 0.0 || alpha < 0.0 || beta < 0.0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "omega/alpha/beta must be >= 0",
            ));
        }
        Ok(Self { inner: CoreGarch::new(omega, alpha, beta) })
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
    fn variance(&self) -> f64 {
        self.inner.variance()
    }
    fn reset(&mut self) {
        ScalarIndicator::reset(&mut self.inner);
    }
    fn __repr__(&self) -> String {
        "Garch()".to_string()
    }
}

macro_rules! scalar_period_indicator {
    ($name:ident, $core:ty, $doc:literal) => {
        #[doc = $doc]
        #[pyclass(module = "qstream")]
        pub struct $name {
            inner: $core,
        }
        #[pymethods]
        impl $name {
            #[new]
            fn new(period: usize) -> PyResult<Self> {
                chk_pos(period, "period")?;
                Ok(Self { inner: <$core>::new(period) })
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
                stringify!($name).to_string()
            }
        }
    };
}

scalar_period_indicator!(
    RealizedVolatility,
    CoreRealizedVol,
    "Realized volatility: sqrt of summed squared returns over a window."
);
scalar_period_indicator!(
    RollingVolatility,
    CoreRollingVol,
    "Rolling volatility: sample standard deviation of returns over a window."
);
scalar_period_indicator!(
    RollingReturns,
    CoreRollingReturns,
    "Rolling cumulative return over a window."
);

/// EWMA of squared returns (variance), returned as volatility.
#[pyclass(module = "qstream")]
pub struct EwmaVariance {
    inner: CoreEwmaVariance,
}

#[pymethods]
impl EwmaVariance {
    #[new]
    #[pyo3(signature = (alpha = 0.06))]
    fn new(alpha: f64) -> PyResult<Self> {
        if !(alpha > 0.0 && alpha <= 1.0) {
            return Err(pyo3::exceptions::PyValueError::new_err("alpha must be in (0,1]"));
        }
        Ok(Self { inner: CoreEwmaVariance::new(alpha) })
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
        "EwmaVariance()".to_string()
    }
}

macro_rules! ohlc_indicator {
    ($name:ident, $core:ty, $doc:literal) => {
        #[doc = $doc]
        #[pyclass(module = "qstream")]
        pub struct $name {
            inner: $core,
        }
        #[pymethods]
        impl $name {
            #[new]
            fn new(period: usize) -> PyResult<Self> {
                chk_pos(period, "period")?;
                Ok(Self { inner: <$core>::new(period) })
            }
            fn update(
                &mut self,
                open: f64,
                high: f64,
                low: f64,
                close: f64,
            ) -> PyResult<Option<f64>> {
                chk(open)?;
                chk(high)?;
                chk(low)?;
                chk(close)?;
                if low <= 0.0 || open <= 0.0 || close <= 0.0 {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "prices must be positive",
                    ));
                }
                Ok(OhlcIndicator::update(&mut self.inner, open, high, low, close))
            }
            fn reset(&mut self) {
                OhlcIndicator::reset(&mut self.inner);
            }
            fn __repr__(&self) -> String {
                stringify!($name).to_string()
            }
        }
    };
}

ohlc_indicator!(GarmanKlass, CoreGarmanKlass, "Garman-Klass OHLC volatility.");
ohlc_indicator!(
    RogersSatchell,
    CoreRogersSatchell,
    "Rogers-Satchell drift-independent OHLC volatility."
);

/// Parkinson high-low range volatility.
#[pyclass(module = "qstream")]
pub struct Parkinson {
    inner: CoreParkinson,
}

#[pymethods]
impl Parkinson {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        chk_pos(period, "period")?;
        Ok(Self { inner: CoreParkinson::new(period) })
    }
    fn update(&mut self, high: f64, low: f64) -> PyResult<Option<f64>> {
        chk2(high, low)?;
        if low <= 0.0 {
            return Err(pyo3::exceptions::PyValueError::new_err("low must be positive"));
        }
        Ok(PairIndicator::update(&mut self.inner, high, low))
    }
    fn reset(&mut self) {
        PairIndicator::reset(&mut self.inner);
    }
    fn __repr__(&self) -> String {
        "Parkinson(period)".to_string()
    }
}

// ---------------------------------------------------------------------------
// Microstructure
// ---------------------------------------------------------------------------

/// Corwin-Schultz high-low bid-ask spread estimator.
#[pyclass(module = "qstream")]
pub struct CorwinSchultz {
    inner: CoreCorwinSchultz,
}

#[pymethods]
impl CorwinSchultz {
    #[new]
    fn new() -> Self {
        Self { inner: CoreCorwinSchultz::new() }
    }
    fn update(&mut self, high: f64, low: f64) -> PyResult<Option<f64>> {
        chk2(high, low)?;
        if low <= 0.0 {
            return Err(pyo3::exceptions::PyValueError::new_err("low must be positive"));
        }
        Ok(PairIndicator::update(&mut self.inner, high, low))
    }
    fn reset(&mut self) {
        PairIndicator::reset(&mut self.inner);
    }
    fn __repr__(&self) -> String {
        "CorwinSchultz()".to_string()
    }
}

/// Abdi-Ranaldo closing-price spread estimator.
#[pyclass(module = "qstream")]
pub struct AbdiRanaldo {
    inner: CoreAbdiRanaldo,
}

#[pymethods]
impl AbdiRanaldo {
    #[new]
    #[pyo3(signature = (period = 20))]
    fn new(period: usize) -> PyResult<Self> {
        if period < 2 {
            return Err(pyo3::exceptions::PyValueError::new_err("period must be >= 2"));
        }
        Ok(Self { inner: CoreAbdiRanaldo::new(period) })
    }
    fn update(&mut self, high: f64, low: f64, close: f64) -> PyResult<Option<f64>> {
        chk(high)?;
        chk(low)?;
        chk(close)?;
        if close <= 0.0 {
            return Err(pyo3::exceptions::PyValueError::new_err("close must be positive"));
        }
        Ok(HlcIndicator::update(&mut self.inner, high, low, close))
    }
    fn reset(&mut self) {
        HlcIndicator::reset(&mut self.inner);
    }
    fn __repr__(&self) -> String {
        "AbdiRanaldo()".to_string()
    }
}

/// Glosten-Milgrom quote/adverse-selection tracker.
#[pyclass(module = "qstream")]
pub struct GlostenMilgrom {
    inner: CoreGlostenMilgrom,
}

#[pymethods]
impl GlostenMilgrom {
    #[new]
    fn new() -> Self {
        Self { inner: CoreGlostenMilgrom::new() }
    }
    fn update(&mut self, trade_price: f64, bid: f64, ask: f64) -> PyResult<f64> {
        chk(trade_price)?;
        chk(bid)?;
        chk(ask)?;
        Ok(self.inner.update(trade_price, bid, ask))
    }
    #[getter]
    fn price_impact(&self) -> f64 {
        self.inner.price_impact()
    }
    #[getter]
    fn spread(&self) -> f64 {
        self.inner.spread()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "GlostenMilgrom()".to_string()
    }
}

// ---------------------------------------------------------------------------
// Tail risk
// ---------------------------------------------------------------------------

/// Historical Value-at-Risk over a rolling window (positive loss magnitude).
#[pyclass(module = "qstream")]
pub struct ValueAtRisk {
    inner: CoreVaR,
}

#[pymethods]
impl ValueAtRisk {
    #[new]
    #[pyo3(signature = (period = 252, p = 0.05))]
    fn new(period: usize, p: f64) -> PyResult<Self> {
        chk_pos(period, "period")?;
        if !(p > 0.0 && p < 1.0) {
            return Err(pyo3::exceptions::PyValueError::new_err("p must be in (0,1)"));
        }
        Ok(Self { inner: CoreVaR::new(period, p) })
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
        "ValueAtRisk()".to_string()
    }
}

/// Conditional VaR / Expected Shortfall over a rolling window.
#[pyclass(module = "qstream")]
pub struct ConditionalValueAtRisk {
    inner: CoreCVaR,
}

#[pymethods]
impl ConditionalValueAtRisk {
    #[new]
    #[pyo3(signature = (period = 252, p = 0.05))]
    fn new(period: usize, p: f64) -> PyResult<Self> {
        chk_pos(period, "period")?;
        if !(p > 0.0 && p < 1.0) {
            return Err(pyo3::exceptions::PyValueError::new_err("p must be in (0,1)"));
        }
        Ok(Self { inner: CoreCVaR::new(period, p) })
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
        "ConditionalValueAtRisk()".to_string()
    }
}

/// Cornish-Fisher modified VaR from rolling moments.
#[pyclass(module = "qstream")]
pub struct CornishFisherVaR {
    inner: CoreCornishFisher,
}

#[pymethods]
impl CornishFisherVaR {
    #[new]
    #[pyo3(signature = (period = 252, confidence = 0.95))]
    fn new(period: usize, confidence: f64) -> PyResult<Self> {
        chk_pos(period, "period")?;
        if !(confidence > 0.0 && confidence < 1.0) {
            return Err(pyo3::exceptions::PyValueError::new_err("confidence must be in (0,1)"));
        }
        Ok(Self { inner: CoreCornishFisher::new(period, confidence) })
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
        "CornishFisherVaR()".to_string()
    }
}

/// Conditional Drawdown-at-Risk from a rolling wealth curve.
#[pyclass(module = "qstream")]
pub struct ConditionalDrawdownAtRisk {
    inner: CoreCDaR,
}

#[pymethods]
impl ConditionalDrawdownAtRisk {
    #[new]
    #[pyo3(signature = (window = 252, alpha = 0.95))]
    fn new(window: usize, alpha: f64) -> PyResult<Self> {
        chk_pos(window, "window")?;
        if !(alpha > 0.0 && alpha < 1.0) {
            return Err(pyo3::exceptions::PyValueError::new_err("alpha must be in (0,1)"));
        }
        Ok(Self { inner: CoreCDaR::new(window, alpha) })
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
    fn max_drawdown(&self) -> f64 {
        self.inner.max_drawdown()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "ConditionalDrawdownAtRisk()".to_string()
    }
}

/// Rachev ratio (upper-tail reward vs lower-tail risk).
#[pyclass(module = "qstream")]
pub struct RachevRatio {
    inner: CoreRachev,
}

#[pymethods]
impl RachevRatio {
    #[new]
    #[pyo3(signature = (period = 252, p = 0.05))]
    fn new(period: usize, p: f64) -> PyResult<Self> {
        chk_pos(period, "period")?;
        if !(p > 0.0 && p < 1.0) {
            return Err(pyo3::exceptions::PyValueError::new_err("p must be in (0,1)"));
        }
        Ok(Self { inner: CoreRachev::new(period, p) })
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
        "RachevRatio()".to_string()
    }
}

// ---------------------------------------------------------------------------
// Factor models
// ---------------------------------------------------------------------------

fn period_to_lambda(period: usize) -> PyResult<f64> {
    if period < 2 {
        return Err(pyo3::exceptions::PyValueError::new_err("period must be >= 2"));
    }
    Ok(1.0 - 1.0 / period as f64)
}

/// Fama-French three-factor model (market, SMB, HML).
#[pyclass(module = "qstream")]
pub struct FamaFrench3 {
    inner: CoreFactorModel,
}

#[pymethods]
impl FamaFrench3 {
    #[new]
    #[pyo3(signature = (period = 252))]
    fn new(period: usize) -> PyResult<Self> {
        let lambda = period_to_lambda(period)?;
        Ok(Self { inner: CoreFactorModel::fama_french3(lambda) })
    }
    #[pyo3(signature = (asset_return, market, smb, hml, risk_free = 0.0))]
    fn update(
        &mut self,
        asset_return: f64,
        market: f64,
        smb: f64,
        hml: f64,
        risk_free: f64,
    ) -> PyResult<FactorResult> {
        chk(asset_return)?;
        chk2(market, smb)?;
        chk2(hml, risk_free)?;
        let out = self
            .inner
            .update(asset_return, &[market, smb, hml], risk_free);
        Ok(out.into())
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "FamaFrench3()".to_string()
    }
}

/// Fama-French five-factor model (market, SMB, HML, RMW, CMA).
#[pyclass(module = "qstream")]
pub struct FamaFrench5 {
    inner: CoreFactorModel,
}

#[pymethods]
impl FamaFrench5 {
    #[new]
    #[pyo3(signature = (period = 252))]
    fn new(period: usize) -> PyResult<Self> {
        let lambda = period_to_lambda(period)?;
        Ok(Self { inner: CoreFactorModel::fama_french5(lambda) })
    }
    #[pyo3(signature = (asset_return, market, smb, hml, rmw, cma, risk_free = 0.0))]
    fn update(
        &mut self,
        asset_return: f64,
        market: f64,
        smb: f64,
        hml: f64,
        rmw: f64,
        cma: f64,
        risk_free: f64,
    ) -> PyResult<FactorResult> {
        chk(asset_return)?;
        chk_slice(&[market, smb, hml, rmw, cma, risk_free])?;
        let out = self
            .inner
            .update(asset_return, &[market, smb, hml, rmw, cma], risk_free);
        Ok(out.into())
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "FamaFrench5()".to_string()
    }
}

/// Carhart four-factor model (market, SMB, HML, momentum).
#[pyclass(module = "qstream")]
pub struct Carhart4 {
    inner: CoreFactorModel,
}

#[pymethods]
impl Carhart4 {
    #[new]
    #[pyo3(signature = (period = 252))]
    fn new(period: usize) -> PyResult<Self> {
        let lambda = period_to_lambda(period)?;
        Ok(Self { inner: CoreFactorModel::carhart4(lambda) })
    }
    #[pyo3(signature = (asset_return, market, smb, hml, mom, risk_free = 0.0))]
    fn update(
        &mut self,
        asset_return: f64,
        market: f64,
        smb: f64,
        hml: f64,
        mom: f64,
        risk_free: f64,
    ) -> PyResult<FactorResult> {
        chk(asset_return)?;
        chk_slice(&[market, smb, hml, mom, risk_free])?;
        let out = self
            .inner
            .update(asset_return, &[market, smb, hml, mom], risk_free);
        Ok(out.into())
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "Carhart4()".to_string()
    }
}

/// Generic N-factor online regression.
#[pyclass(module = "qstream")]
pub struct MultiFactorModel {
    inner: CoreFactorModel,
}

#[pymethods]
impl MultiFactorModel {
    #[new]
    #[pyo3(signature = (n_factors, period = 252))]
    fn new(n_factors: usize, period: usize) -> PyResult<Self> {
        chk_pos(n_factors, "n_factors")?;
        let lambda = period_to_lambda(period)?;
        Ok(Self { inner: CoreFactorModel::new(n_factors, lambda) })
    }
    #[pyo3(signature = (asset_return, factors, risk_free = 0.0))]
    fn update(
        &mut self,
        asset_return: f64,
        factors: Vec<f64>,
        risk_free: f64,
    ) -> PyResult<FactorResult> {
        chk(asset_return)?;
        chk(risk_free)?;
        chk_slice(&factors)?;
        let out = self.inner.update(asset_return, &factors, risk_free);
        Ok(out.into())
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "MultiFactorModel()".to_string()
    }
}

/// Treynor-Mazuy market-timing model.
#[pyclass(module = "qstream")]
pub struct TreynorMazuy {
    inner: CoreTreynorMazuy,
}

#[pymethods]
impl TreynorMazuy {
    #[new]
    #[pyo3(signature = (period = 252))]
    fn new(period: usize) -> PyResult<Self> {
        let lambda = period_to_lambda(period)?;
        Ok(Self { inner: CoreTreynorMazuy::new(lambda) })
    }
    #[pyo3(signature = (asset_return, market, risk_free = 0.0))]
    fn update(
        &mut self,
        asset_return: f64,
        market: f64,
        risk_free: f64,
    ) -> PyResult<FactorResult> {
        chk(asset_return)?;
        chk2(market, risk_free)?;
        let out = self.inner.update(asset_return, market, risk_free);
        Ok(out.into())
    }
    #[getter]
    fn gamma(&self) -> f64 {
        self.inner.gamma()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "TreynorMazuy()".to_string()
    }
}

/// CAPM alpha/beta via rolling excess-return regression.
#[pyclass(module = "qstream")]
pub struct Capm {
    inner: CoreCapm,
}

#[pymethods]
impl Capm {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        chk_pos(period, "period")?;
        Ok(Self { inner: CoreCapm::new(period) })
    }
    #[pyo3(signature = (asset_return, benchmark_return, risk_free = 0.0))]
    fn update(
        &mut self,
        asset_return: f64,
        benchmark_return: f64,
        risk_free: f64,
    ) -> PyResult<Option<FactorResult>> {
        chk(asset_return)?;
        chk2(benchmark_return, risk_free)?;
        Ok(self.inner.update(asset_return, benchmark_return, risk_free).map(
            |(alpha, beta, r2)| FactorResult {
                alpha,
                betas: vec![beta],
                r2,
                residual_variance: f64::NAN,
            },
        ))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "Capm()".to_string()
    }
}

/// Jensen's alpha (intercept of excess-return regression).
#[pyclass(module = "qstream")]
pub struct JensenAlpha {
    inner: CoreCapm,
}

#[pymethods]
impl JensenAlpha {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        chk_pos(period, "period")?;
        Ok(Self { inner: CoreCapm::new(period) })
    }
    #[pyo3(signature = (asset_return, benchmark_return, risk_free = 0.0))]
    fn update(
        &mut self,
        asset_return: f64,
        benchmark_return: f64,
        risk_free: f64,
    ) -> PyResult<Option<f64>> {
        chk(asset_return)?;
        chk2(benchmark_return, risk_free)?;
        Ok(self
            .inner
            .update(asset_return, benchmark_return, risk_free)
            .map(|(alpha, _, _)| alpha))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "JensenAlpha()".to_string()
    }
}

/// Rolling beta of an asset versus a benchmark.
#[pyclass(module = "qstream")]
pub struct RollingBeta {
    inner: CoreRollingBeta,
}

#[pymethods]
impl RollingBeta {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        chk_pos(period, "period")?;
        Ok(Self { inner: CoreRollingBeta::new(period) })
    }
    fn update(&mut self, asset_return: f64, market_return: f64) -> PyResult<Option<f64>> {
        chk2(asset_return, market_return)?;
        Ok(self.inner.update(asset_return, market_return))
    }
    #[getter]
    fn correlation(&self) -> f64 {
        self.inner.correlation()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "RollingBeta()".to_string()
    }
}

/// Rolling-window beta stability (standard deviation of the rolling beta).
#[pyclass(module = "qstream")]
pub struct RollingBetaStability {
    inner: CoreBetaStability,
}

#[pymethods]
impl RollingBetaStability {
    #[new]
    #[pyo3(signature = (beta_window = 60, stability_window = 60))]
    fn new(beta_window: usize, stability_window: usize) -> PyResult<Self> {
        chk_pos(beta_window, "beta_window")?;
        chk_pos(stability_window, "stability_window")?;
        Ok(Self {
            inner: CoreBetaStability::new(beta_window, stability_window),
        })
    }
    fn update(&mut self, asset_return: f64, market_return: f64) -> PyResult<Option<f64>> {
        chk2(asset_return, market_return)?;
        Ok(self.inner.update(asset_return, market_return))
    }
    #[getter]
    fn beta(&self) -> f64 {
        self.inner.beta()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "RollingBetaStability()".to_string()
    }
}

/// Tracking error: rolling standard deviation of active returns.
#[pyclass(module = "qstream")]
pub struct TrackingError {
    inner: CoreTrackingError,
}

#[pymethods]
impl TrackingError {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        chk_pos(period, "period")?;
        Ok(Self { inner: CoreTrackingError::new(period) })
    }
    fn update(&mut self, asset_return: f64, benchmark_return: f64) -> PyResult<Option<f64>> {
        chk2(asset_return, benchmark_return)?;
        Ok(self.inner.update(asset_return, benchmark_return))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "TrackingError()".to_string()
    }
}

// ---------------------------------------------------------------------------
// Performance
// ---------------------------------------------------------------------------

/// Rolling Sharpe ratio.
#[pyclass(module = "qstream")]
pub struct SharpeRatio {
    inner: CoreSharpe,
}

#[pymethods]
impl SharpeRatio {
    #[new]
    #[pyo3(signature = (period = 252, risk_free = 0.0))]
    fn new(period: usize, risk_free: f64) -> PyResult<Self> {
        chk_pos(period, "period")?;
        chk(risk_free)?;
        Ok(Self { inner: CoreSharpe::new(period, risk_free) })
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
        "SharpeRatio()".to_string()
    }
}

/// Sortino ratio with a minimum acceptable return (MAR).
#[pyclass(module = "qstream")]
pub struct SortinoRatio {
    inner: CoreSortino,
}

#[pymethods]
impl SortinoRatio {
    #[new]
    #[pyo3(signature = (period = 252, mar = 0.0))]
    fn new(period: usize, mar: f64) -> PyResult<Self> {
        chk_pos(period, "period")?;
        chk(mar)?;
        Ok(Self { inner: CoreSortino::new(period, mar) })
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
    fn downside_deviation(&self) -> f64 {
        self.inner.downside_deviation()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "SortinoRatio()".to_string()
    }
}

/// Gain-loss (Bernardo-Ledoit) ratio.
#[pyclass(module = "qstream")]
pub struct GainLossRatio {
    inner: CoreGainLoss,
}

#[pymethods]
impl GainLossRatio {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        chk_pos(period, "period")?;
        Ok(Self { inner: CoreGainLoss::new(period) })
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
        "GainLossRatio()".to_string()
    }
}

/// Omega ratio above a threshold.
#[pyclass(module = "qstream")]
pub struct OmegaRatio {
    inner: CoreOmega,
}

#[pymethods]
impl OmegaRatio {
    #[new]
    #[pyo3(signature = (period = 252, threshold = 0.0))]
    fn new(period: usize, threshold: f64) -> PyResult<Self> {
        chk_pos(period, "period")?;
        chk(threshold)?;
        Ok(Self { inner: CoreOmega::new(period, threshold) })
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
        "OmegaRatio()".to_string()
    }
}

/// Up/down capture ratios versus a benchmark.
#[pyclass(module = "qstream")]
pub struct CaptureRatios {
    inner: CoreCapture,
}

#[pymethods]
impl CaptureRatios {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        chk_pos(period, "period")?;
        Ok(Self { inner: CoreCapture::new(period) })
    }
    fn update(&mut self, asset_return: f64, benchmark_return: f64) -> PyResult<Option<CaptureResult>> {
        chk2(asset_return, benchmark_return)?;
        Ok(self.inner.update(asset_return, benchmark_return).map(|(up, down)| {
            CaptureResult { up_capture: up, down_capture: down }
        }))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "CaptureRatios()".to_string()
    }
}

/// Conditional Sharpe ratio (asset Sharpe conditioned on benchmark direction).
#[pyclass(module = "qstream")]
pub struct ConditionalSharpe {
    inner: CoreCondSharpe,
}

#[pymethods]
impl ConditionalSharpe {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        chk_pos(period, "period")?;
        Ok(Self { inner: CoreCondSharpe::new(period) })
    }
    fn update(&mut self, asset_return: f64, benchmark_return: f64) -> PyResult<()> {
        chk2(asset_return, benchmark_return)?;
        self.inner.update(asset_return, benchmark_return);
        Ok(())
    }
    #[getter]
    fn up_sharpe(&mut self) -> f64 {
        self.inner.up_sharpe()
    }
    #[getter]
    fn down_sharpe(&mut self) -> f64 {
        self.inner.down_sharpe()
    }
    #[getter]
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "ConditionalSharpe()".to_string()
    }
}

/// Deflated Sharpe Ratio (Bailey & Lopez de Prado).
#[pyclass(module = "qstream")]
pub struct DeflatedSharpeRatio {
    inner: CoreDeflatedSharpe,
}

#[pymethods]
impl DeflatedSharpeRatio {
    #[new]
    #[pyo3(signature = (period = 252, n_trials = 1, risk_free = 0.0))]
    fn new(period: usize, n_trials: usize, risk_free: f64) -> PyResult<Self> {
        chk_pos(period, "period")?;
        chk_pos(n_trials, "n_trials")?;
        chk(risk_free)?;
        Ok(Self { inner: CoreDeflatedSharpe::new(period, n_trials, risk_free) })
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
        "DeflatedSharpeRatio()".to_string()
    }
}

/// Lo (2002) autocorrelation-adjusted Sharpe ratio.
#[pyclass(module = "qstream")]
pub struct LoAutocorrelationSharpe {
    inner: CoreLoSharpe,
}

#[pymethods]
impl LoAutocorrelationSharpe {
    #[new]
    #[pyo3(signature = (period = 252, risk_free = 0.0))]
    fn new(period: usize, risk_free: f64) -> PyResult<Self> {
        chk_pos(period, "period")?;
        chk(risk_free)?;
        Ok(Self { inner: CoreLoSharpe::new(period, risk_free) })
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
    fn autocorrelation(&self) -> f64 {
        self.inner.autocorrelation()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "LoAutocorrelationSharpe()".to_string()
    }
}

// ---------------------------------------------------------------------------
// Portfolio
// ---------------------------------------------------------------------------

/// Weighted portfolio return stream.
#[pyclass(module = "qstream")]
pub struct PortfolioReturns {
    inner: CorePortfolioReturns,
}

#[pymethods]
impl PortfolioReturns {
    #[new]
    fn new(n_assets: usize) -> PyResult<Self> {
        chk_pos(n_assets, "n_assets")?;
        Ok(Self { inner: CorePortfolioReturns::new(n_assets) })
    }
    fn update(&mut self, returns: Vec<f64>, weights: Vec<f64>) -> PyResult<f64> {
        chk_slice(&returns)?;
        chk_slice(&weights)?;
        Ok(self.inner.update(&returns, &weights))
    }
    #[getter]
    fn cumulative(&self) -> f64 {
        self.inner.cumulative()
    }
    #[getter]
    fn mean(&self) -> f64 {
        self.inner.mean()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "PortfolioReturns()".to_string()
    }
}

/// Aggregate portfolio duration.
#[pyclass(module = "qstream")]
pub struct PortfolioDuration {
    inner: CorePortfolioDuration,
}

#[pymethods]
impl PortfolioDuration {
    #[new]
    fn new(n_assets: usize) -> PyResult<Self> {
        chk_pos(n_assets, "n_assets")?;
        Ok(Self { inner: CorePortfolioDuration::new(n_assets) })
    }
    fn update(&mut self, durations: Vec<f64>, weights: Vec<f64>) -> PyResult<f64> {
        chk_slice(&durations)?;
        chk_slice(&weights)?;
        Ok(self.inner.update(&durations, &weights))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "PortfolioDuration()".to_string()
    }
}

/// EWMA multivariate covariance matrix.
#[pyclass(module = "qstream")]
pub struct EwmaCovarianceMatrix {
    inner: CoreEwmaCov,
}

#[pymethods]
impl EwmaCovarianceMatrix {
    #[new]
    #[pyo3(signature = (n_assets, lambda = 0.94))]
    fn new(n_assets: usize, lambda: f64) -> PyResult<Self> {
        chk_pos(n_assets, "n_assets")?;
        if !(lambda > 0.0 && lambda < 1.0) {
            return Err(pyo3::exceptions::PyValueError::new_err("lambda must be in (0,1)"));
        }
        Ok(Self { inner: CoreEwmaCov::new(n_assets, lambda) })
    }
    fn update(&mut self, returns: Vec<f64>) -> PyResult<()> {
        chk_slice(&returns)?;
        self.inner.update(&returns);
        Ok(())
    }
    /// Portfolio variance for a weight vector.
    fn portfolio_variance(&self, weights: Vec<f64>) -> f64 {
        self.inner.portfolio_variance(&weights)
    }
    /// Full covariance as a flat row-major list.
    fn to_list(&self) -> Vec<f64> {
        self.inner.as_slice().to_vec()
    }
    #[getter]
    fn dim(&self) -> usize {
        self.inner.dim()
    }
    #[getter]
    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "EwmaCovarianceMatrix()".to_string()
    }
}

/// Component & marginal VaR from a streaming EWMA covariance.
#[pyclass(module = "qstream")]
pub struct ComponentMarginalVaR {
    inner: CoreComponentVaR,
}

#[pymethods]
impl ComponentMarginalVaR {
    #[new]
    #[pyo3(signature = (n_assets, lambda = 0.94, confidence = 0.95))]
    fn new(n_assets: usize, lambda: f64, confidence: f64) -> PyResult<Self> {
        chk_pos(n_assets, "n_assets")?;
        if !(lambda > 0.0 && lambda < 1.0) {
            return Err(pyo3::exceptions::PyValueError::new_err("lambda must be in (0,1)"));
        }
        if !(confidence > 0.0 && confidence < 1.0) {
            return Err(pyo3::exceptions::PyValueError::new_err("confidence must be in (0,1)"));
        }
        Ok(Self { inner: CoreComponentVaR::new(n_assets, lambda, confidence) })
    }
    fn update(&mut self, returns: Vec<f64>) -> PyResult<()> {
        chk_slice(&returns)?;
        self.inner.update(&returns);
        Ok(())
    }
    fn compute(&mut self, weights: Vec<f64>) -> PyResult<Option<ComponentVaRResult>> {
        chk_slice(&weights)?;
        make_component_var(&mut self.inner, &weights)
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "ComponentMarginalVaR()".to_string()
    }
}

/// Equal-risk-contribution (risk parity) weights from a covariance matrix.
#[pyfunction]
#[pyo3(signature = (covariance, n_assets, iters = 200))]
pub fn risk_parity_weights(
    covariance: Vec<f64>,
    n_assets: usize,
    iters: usize,
) -> PyResult<Vec<f64>> {
    chk_slice(&covariance)?;
    if covariance.len() != n_assets * n_assets {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "covariance length must equal n_assets^2",
        ));
    }
    Ok(CoreRiskParity::solve(&covariance, n_assets, iters))
}

// ---------------------------------------------------------------------------
// Derivatives
// ---------------------------------------------------------------------------

/// Black-Scholes implied volatility (Newton-Raphson with bisection safeguard).
#[pyclass(module = "qstream")]
pub struct ImpliedVolatility {
    inner: deriv::ImpliedVolatility,
}

#[pymethods]
impl ImpliedVolatility {
    #[new]
    fn new() -> Self {
        Self { inner: deriv::ImpliedVolatility::new() }
    }
    #[pyo3(signature = (market_price, spot, strike, ttm, rate = 0.0, is_call = true))]
    fn update(
        &mut self,
        market_price: f64,
        spot: f64,
        strike: f64,
        ttm: f64,
        rate: f64,
        is_call: bool,
    ) -> PyResult<Option<f64>> {
        chk(market_price)?;
        chk2(spot, strike)?;
        chk2(ttm, rate)?;
        Ok(self.inner.update(market_price, spot, strike, ttm, rate, is_call))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "ImpliedVolatility()".to_string()
    }
}

/// Black-Scholes European option price (convenience function).
#[pyfunction]
#[pyo3(signature = (spot, strike, ttm, rate, sigma, is_call = true))]
pub fn bs_price(spot: f64, strike: f64, ttm: f64, rate: f64, sigma: f64, is_call: bool) -> PyResult<f64> {
    chk2(spot, strike)?;
    chk2(ttm, rate)?;
    chk(sigma)?;
    Ok(deriv::bs_price(spot, strike, ttm, rate, sigma, is_call))
}

/// VIX futures term-structure shape from one snapshot.
#[pyclass(module = "qstream")]
pub struct VixTermStructure;

#[pymethods]
impl VixTermStructure {
    #[new]
    fn new() -> Self {
        Self
    }
    fn update(
        &mut self,
        spot_vix: f64,
        futures: Vec<f64>,
        maturities: Vec<f64>,
    ) -> PyResult<Option<TermStructureResult>> {
        chk(spot_vix)?;
        chk_slice(&futures)?;
        chk_slice(&maturities)?;
        Ok(deriv::term_structure(spot_vix, &futures, &maturities).map(|t| t.into()))
    }
    fn __repr__(&self) -> String {
        "VixTermStructure()".to_string()
    }
}

/// Realized-variance tracker for variance swaps.
#[pyclass(module = "qstream")]
pub struct VarianceSwap {
    inner: CoreVarianceSwap,
}

#[pymethods]
impl VarianceSwap {
    #[new]
    fn new(period: usize) -> PyResult<Self> {
        chk_pos(period, "period")?;
        Ok(Self { inner: CoreVarianceSwap::new(period) })
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
    fn realized_variance(&self) -> f64 {
        self.inner.realized_variance()
    }
    #[getter]
    fn realized_volatility(&self) -> f64 {
        self.inner.realized_volatility()
    }
    #[pyo3(signature = (strike_variance, notional = 1.0))]
    fn pnl(&self, strike_variance: f64, notional: f64) -> f64 {
        self.inner.pnl(strike_variance, notional)
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn __repr__(&self) -> String {
        "VarianceSwap()".to_string()
    }
}
