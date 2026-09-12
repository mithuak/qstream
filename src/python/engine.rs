//! FeatureEngine: Rust-owned grouped execution. One Python call updates many
//! indicators, amortizing PyO3 crossing overhead (design sections 13-14).

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::core::traits::ScalarIndicator;
use crate::finance::performance::{
    GainLossRatio, LoAutocorrelationSharpe, SharpeRatio, SortinoRatio,
};
use crate::finance::risk::ValueAtRisk;
use crate::finance::volatility::{
    EwmaVolatility, Garch11, RealizedVolatility, RollingReturns, RollingVolatility,
};
use crate::signal::filters::{
    AdaptiveNotchFilter, KolmogorovZurbenko, SavitzkyGolay, WienerFilter,
};
use crate::signal::kalman::{
    AdaptiveKalman, AlphaBetaTracker, ConstantVelocityKalman, LmsFilter, RlsFilter,
};
use crate::signal::regime::{PageHinkley, TeagerKaiser, ZeroCrossingRate};

/// Input channel a feature is bound to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Channel {
    Open,
    High,
    Low,
    Close,
    Volume,
    Return,
}

impl Channel {
    fn parse(s: &str) -> Option<Channel> {
        match s.to_ascii_lowercase().as_str() {
            "open" => Some(Channel::Open),
            "high" => Some(Channel::High),
            "low" => Some(Channel::Low),
            "close" | "price" => Some(Channel::Close),
            "volume" => Some(Channel::Volume),
            "return" | "returns" => Some(Channel::Return),
            _ => None,
        }
    }
}

/// A single scalar feature fed from one channel.
enum ScalarFeature {
    EwmaVol(EwmaVolatility),
    Garch(Garch11),
    RealizedVol(RealizedVolatility),
    RollingVol(RollingVolatility),
    RollingReturns(RollingReturns),
    Sharpe(SharpeRatio),
    Sortino(SortinoRatio),
    LoSharpe(LoAutocorrelationSharpe),
    GainLoss(GainLossRatio),
    ValueAtRisk(ValueAtRisk),
    PageHinkley(PageHinkley),
    TeagerKaiser(TeagerKaiser),
    ZeroCrossing(ZeroCrossingRate),
    SavitzkyGolay(SavitzkyGolay),
    KolmogorovZurbenko(KolmogorovZurbenko),
    Wiener(WienerFilter),
    AdaptiveNotch(AdaptiveNotchFilter),
    AlphaBeta(AlphaBetaTracker),
    Kalman(ConstantVelocityKalman),
    AdaptiveKalman(AdaptiveKalman),
    Lms(LmsFilter),
    Rls(RlsFilter),
}

impl ScalarFeature {
    fn update(&mut self, x: f64) -> Option<f64> {
        match self {
            ScalarFeature::EwmaVol(f) => ScalarIndicator::update(f, x),
            ScalarFeature::Garch(f) => ScalarIndicator::update(f, x),
            ScalarFeature::RealizedVol(f) => ScalarIndicator::update(f, x),
            ScalarFeature::RollingVol(f) => ScalarIndicator::update(f, x),
            ScalarFeature::RollingReturns(f) => ScalarIndicator::update(f, x),
            ScalarFeature::Sharpe(f) => f.update(x),
            ScalarFeature::Sortino(f) => f.update(x),
            ScalarFeature::LoSharpe(f) => f.update(x),
            ScalarFeature::GainLoss(f) => f.update(x),
            ScalarFeature::ValueAtRisk(f) => f.update(x),
            ScalarFeature::PageHinkley(f) => Some(f.update(x).score),
            ScalarFeature::TeagerKaiser(f) => ScalarIndicator::update(f, x),
            ScalarFeature::ZeroCrossing(f) => f.update(x),
            ScalarFeature::SavitzkyGolay(f) => ScalarIndicator::update(f, x),
            ScalarFeature::KolmogorovZurbenko(f) => ScalarIndicator::update(f, x),
            ScalarFeature::Wiener(f) => ScalarIndicator::update(f, x),
            ScalarFeature::AdaptiveNotch(f) => Some(f.update(x).frequency),
            ScalarFeature::AlphaBeta(f) => Some(f.update(x).value),
            ScalarFeature::Kalman(f) => Some(f.update(x).value),
            ScalarFeature::AdaptiveKalman(f) => Some(f.update(x).value),
            ScalarFeature::Lms(f) => ScalarIndicator::update(f, x),
            ScalarFeature::Rls(f) => ScalarIndicator::update(f, x),
        }
    }
}

fn build_feature(kind: &str, param: Option<f64>) -> PyResult<ScalarFeature> {
    let p = |d: f64| param.unwrap_or(d);
    Ok(match kind.to_ascii_lowercase().as_str() {
        "ewma_volatility" | "ewmavol" => {
            ScalarFeature::EwmaVol(EwmaVolatility::new(1.0 - p(0.06).clamp(1e-6, 1.0 - 1e-6)))
        }
        "garch" => ScalarFeature::Garch(Garch11::default_params()),
        "realized_volatility" => ScalarFeature::RealizedVol(RealizedVolatility::new(p(20.0) as usize)),
        "rolling_volatility" => ScalarFeature::RollingVol(RollingVolatility::new((p(20.0) as usize).max(2))),
        "rolling_returns" => ScalarFeature::RollingReturns(RollingReturns::new(p(20.0) as usize)),
        "sharpe" => ScalarFeature::Sharpe(SharpeRatio::new(p(252.0) as usize, 0.0)),
        "sortino" => ScalarFeature::Sortino(SortinoRatio::new(p(252.0) as usize, 0.0)),
        "lo_sharpe" | "lo_autocorrelation_sharpe" => {
            ScalarFeature::LoSharpe(LoAutocorrelationSharpe::new(p(252.0) as usize, 0.0))
        }
        "gain_loss" => ScalarFeature::GainLoss(GainLossRatio::new(p(252.0) as usize)),
        "value_at_risk" | "var" => ScalarFeature::ValueAtRisk(ValueAtRisk::new(p(252.0) as usize, 0.05)),
        "page_hinkley" => ScalarFeature::PageHinkley(PageHinkley::new(0.005, p(1.0))),
        "teager_kaiser" => ScalarFeature::TeagerKaiser(TeagerKaiser::new()),
        "zero_crossing" => ScalarFeature::ZeroCrossing(ZeroCrossingRate::new((p(32.0) as usize).max(2), 0.0)),
        "savgol" | "savitzky_golay" => {
            let mut w = (p(9.0) as usize).max(3);
            if w % 2 == 0 {
                w += 1;
            }
            ScalarFeature::SavitzkyGolay(SavitzkyGolay::new(w, 2))
        }
        "kz" | "kolmogorov_zurbenko" => {
            ScalarFeature::KolmogorovZurbenko(KolmogorovZurbenko::new(p(11.0) as usize, 3))
        }
        "wiener" => ScalarFeature::Wiener(WienerFilter::new(p(9.0) as usize, 32)),
        "adaptive_notch" => ScalarFeature::AdaptiveNotch(AdaptiveNotchFilter::new(0.95, 1e-3, 0.05)),
        "alpha_beta" => ScalarFeature::AlphaBeta(AlphaBetaTracker::new(0.5, 0.1, 1.0)),
        "kalman" => ScalarFeature::Kalman(ConstantVelocityKalman::new(1.0, 1e-3, p(1.0), 1.0)),
        "adaptive_kalman" => ScalarFeature::AdaptiveKalman(AdaptiveKalman::new(1e-3, 1e-2, 0.05)),
        "lms" => ScalarFeature::Lms(LmsFilter::new((p(4.0) as usize).max(1), 0.01)),
        "rls" => ScalarFeature::Rls(RlsFilter::new((p(4.0) as usize).max(1), 1.0)),
        other => {
            return Err(PyValueError::new_err(format!("unknown feature kind: {other}")));
        }
    })
}

/// Grouped feature engine. Configure with parallel lists of names, kinds,
/// channels, and optional numeric parameters. Each `update` call feeds every
/// feature its channel value in one Python->Rust crossing.
#[pyclass(module = "qstream")]
pub struct FeatureEngine {
    features: Vec<(String, Channel, ScalarFeature)>,
    prev_close: f64,
    have_prev: bool,
}

#[pymethods]
impl FeatureEngine {
    #[new]
    #[pyo3(signature = (names, kinds, channels, params = None))]
    fn new(
        names: Vec<String>,
        kinds: Vec<String>,
        channels: Vec<String>,
        params: Option<Vec<Option<f64>>>,
    ) -> PyResult<Self> {
        if names.len() != kinds.len() || names.len() != channels.len() {
            return Err(PyValueError::new_err(
                "names, kinds, channels must have equal length",
            ));
        }
        let n = names.len();
        let params = params.unwrap_or_else(|| vec![None; n]);
        if params.len() != n {
            return Err(PyValueError::new_err("params length mismatch"));
        }
        let mut features = Vec::with_capacity(n);
        for i in 0..n {
            let ch = Channel::parse(&channels[i]).ok_or_else(|| {
                PyValueError::new_err(format!("unknown channel: {}", channels[i]))
            })?;
            let feat = build_feature(&kinds[i], params[i])?;
            features.push((names[i].clone(), ch, feat));
        }
        Ok(Self { features, prev_close: 0.0, have_prev: false })
    }

    /// Update all features from an OHLCV bar. The `return` channel is computed
    /// internally from successive closes. Returns values in feature order
    /// (`None` where a feature is still warming up); zip with `.names` to map
    /// back to names. Returning an ordered list (not a dict) avoids allocating
    /// Python string keys on every tick.
    #[pyo3(signature = (open, high, low, close, volume = 0.0))]
    fn update(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> PyResult<Vec<Option<f64>>> {
        let ret = if self.have_prev && self.prev_close != 0.0 {
            close / self.prev_close - 1.0
        } else {
            0.0
        };
        self.prev_close = close;
        self.have_prev = true;
        let mut out = Vec::with_capacity(self.features.len());
        for (_name, ch, feat) in self.features.iter_mut() {
            let x = match ch {
                Channel::Open => open,
                Channel::High => high,
                Channel::Low => low,
                Channel::Close => close,
                Channel::Volume => volume,
                Channel::Return => ret,
            };
            out.push(feat.update(x));
        }
        Ok(out)
    }

    /// Convenience for close-only engines: feeds `value` to every channel.
    fn update_scalar(&mut self, value: f64) -> PyResult<Vec<Option<f64>>> {
        self.update(value, value, value, value, 0.0)
    }

    /// Feature names in the order returned by `update`.
    #[getter]
    fn names(&self) -> Vec<String> {
        self.features.iter().map(|(n, _, _)| n.clone()).collect()
    }

    /// Number of features in the engine.
    #[getter]
    fn len(&self) -> usize {
        self.features.len()
    }

    fn __len__(&self) -> usize {
        self.features.len()
    }

    fn __repr__(&self) -> String {
        format!("FeatureEngine(n_features={})", self.features.len())
    }
}
