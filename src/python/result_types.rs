//! Lightweight PyO3 result objects (design section 9). These wrap compact
//! copied data; they are only created when a computation actually ran.

use pyo3::prelude::*;

use crate::finance::derivatives::TermStructure as CoreTerm;
use crate::finance::factor::FactorOutput as CoreFactor;
use crate::finance::portfolio::ComponentMarginalVaR;
use crate::signal::kalman::StateEstimate as CoreState;
use crate::signal::prediction::PredictionResult as CorePrediction;
use crate::signal::regime::ChangeResult as CoreChange;
use crate::signal::spectral::{SpectralShape as CoreShape, SpectrumResult as CoreSpectrum};
use crate::signal::wavelet::WaveletResult as CoreWavelet;

/// State estimate from a Kalman / state-space filter.
#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct StateEstimate {
    #[pyo3(get)]
    pub value: f64,
    #[pyo3(get)]
    pub velocity: Option<f64>,
    #[pyo3(get)]
    pub covariance: Vec<f64>,
}

impl From<CoreState> for StateEstimate {
    fn from(s: CoreState) -> Self {
        Self { value: s.value, velocity: s.velocity, covariance: s.covariance }
    }
}

#[pymethods]
impl StateEstimate {
    fn __repr__(&self) -> String {
        format!(
            "StateEstimate(value={}, velocity={:?}, covariance=[len {}])",
            self.value,
            self.velocity,
            self.covariance.len()
        )
    }
}

/// Factor-model regression output.
#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct FactorResult {
    #[pyo3(get)]
    pub alpha: f64,
    #[pyo3(get)]
    pub betas: Vec<f64>,
    #[pyo3(get)]
    pub r2: f64,
    #[pyo3(get)]
    pub residual_variance: f64,
}

impl From<CoreFactor> for FactorResult {
    fn from(f: CoreFactor) -> Self {
        Self {
            alpha: f.alpha,
            betas: f.betas,
            r2: f.r2,
            residual_variance: f.residual_variance,
        }
    }
}

#[pymethods]
impl FactorResult {
    fn __repr__(&self) -> String {
        format!(
            "FactorResult(alpha={}, betas={:?}, r2={}, residual_variance={})",
            self.alpha, self.betas, self.r2, self.residual_variance
        )
    }
}

/// One-sided spectral estimate.
#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct SpectrumResult {
    #[pyo3(get)]
    pub frequencies: Vec<f64>,
    #[pyo3(get)]
    pub power: Vec<f64>,
    #[pyo3(get)]
    pub dominant_frequency: Option<f64>,
    #[pyo3(get)]
    pub peak_power: Option<f64>,
}

impl From<CoreSpectrum> for SpectrumResult {
    fn from(s: CoreSpectrum) -> Self {
        Self {
            frequencies: s.frequencies,
            power: s.power,
            dominant_frequency: s.dominant_frequency,
            peak_power: s.peak_power,
        }
    }
}

#[pymethods]
impl SpectrumResult {
    fn __repr__(&self) -> String {
        format!(
            "SpectrumResult(bins={}, dominant_frequency={:?}, peak_power={:?})",
            self.frequencies.len(),
            self.dominant_frequency,
            self.peak_power
        )
    }
}

/// Wavelet-domain result.
#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct WaveletResult {
    #[pyo3(get)]
    pub coefficients: Vec<f64>,
    #[pyo3(get)]
    pub scales: Option<Vec<f64>>,
}

impl From<CoreWavelet> for WaveletResult {
    fn from(w: CoreWavelet) -> Self {
        Self { coefficients: w.coefficients, scales: w.scales }
    }
}

#[pymethods]
impl WaveletResult {
    fn __repr__(&self) -> String {
        format!(
            "WaveletResult(coefficients=[len {}], scales={:?})",
            self.coefficients.len(),
            self.scales.as_ref().map(|s| s.len())
        )
    }
}

/// Change-detection result.
#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct ChangeResult {
    #[pyo3(get)]
    pub changed: bool,
    #[pyo3(get)]
    pub score: f64,
}

impl From<CoreChange> for ChangeResult {
    fn from(c: CoreChange) -> Self {
        Self { changed: c.changed, score: c.score }
    }
}

#[pymethods]
impl ChangeResult {
    fn __repr__(&self) -> String {
        format!("ChangeResult(changed={}, score={})", self.changed, self.score)
    }
}

/// Prediction result (LPC / lattice).
#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct PredictionResult {
    #[pyo3(get)]
    pub prediction: f64,
    #[pyo3(get)]
    pub coefficients: Option<Vec<f64>>,
}

impl From<CorePrediction> for PredictionResult {
    fn from(p: CorePrediction) -> Self {
        Self { prediction: p.prediction, coefficients: Some(p.coefficients) }
    }
}

#[pymethods]
impl PredictionResult {
    fn __repr__(&self) -> String {
        format!(
            "PredictionResult(prediction={}, coefficients={:?})",
            self.prediction,
            self.coefficients.as_ref().map(|c| c.len())
        )
    }
}

/// VIX term-structure shape.
#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct TermStructureResult {
    #[pyo3(get)]
    pub slope: f64,
    #[pyo3(get)]
    pub curvature: f64,
    #[pyo3(get)]
    pub contango: f64,
}

impl From<CoreTerm> for TermStructureResult {
    fn from(t: CoreTerm) -> Self {
        Self { slope: t.slope, curvature: t.curvature, contango: t.contango }
    }
}

#[pymethods]
impl TermStructureResult {
    fn __repr__(&self) -> String {
        format!(
            "TermStructureResult(slope={}, curvature={}, contango={})",
            self.slope, self.curvature, self.contango
        )
    }
}

/// Spectral-shape features.
#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct SpectralShapeResult {
    #[pyo3(get)]
    pub centroid: f64,
    #[pyo3(get)]
    pub bandwidth: f64,
    #[pyo3(get)]
    pub entropy: f64,
    #[pyo3(get)]
    pub rolloff: f64,
    #[pyo3(get)]
    pub flatness: f64,
    #[pyo3(get)]
    pub dominant_frequency: f64,
}

impl From<CoreShape> for SpectralShapeResult {
    fn from(s: CoreShape) -> Self {
        Self {
            centroid: s.centroid,
            bandwidth: s.bandwidth,
            entropy: s.entropy,
            rolloff: s.rolloff,
            flatness: s.flatness,
            dominant_frequency: s.dominant_frequency,
        }
    }
}

#[pymethods]
impl SpectralShapeResult {
    fn __repr__(&self) -> String {
        format!(
            "SpectralShapeResult(centroid={}, bandwidth={}, entropy={}, rolloff={}, flatness={})",
            self.centroid, self.bandwidth, self.entropy, self.rolloff, self.flatness
        )
    }
}

/// Component / marginal VaR result.
#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct ComponentVaRResult {
    #[pyo3(get)]
    pub total_var: f64,
    #[pyo3(get)]
    pub marginal: Vec<f64>,
    #[pyo3(get)]
    pub component: Vec<f64>,
}

#[pymethods]
impl ComponentVaRResult {
    fn __repr__(&self) -> String {
        format!(
            "ComponentVaRResult(total_var={}, n={})",
            self.total_var,
            self.component.len()
        )
    }
}

/// Up/down capture ratios.
#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct CaptureResult {
    #[pyo3(get)]
    pub up_capture: f64,
    #[pyo3(get)]
    pub down_capture: f64,
}

#[pymethods]
impl CaptureResult {
    fn __repr__(&self) -> String {
        format!(
            "CaptureResult(up_capture={}, down_capture={})",
            self.up_capture, self.down_capture
        )
    }
}

/// Frequency estimate from an adaptive cycle filter (e.g. adaptive notch).
#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct FrequencyEstimate {
    #[pyo3(get)]
    pub frequency: f64,
    #[pyo3(get)]
    pub filtered: Option<f64>,
}

#[pymethods]
impl FrequencyEstimate {
    fn __repr__(&self) -> String {
        format!(
            "FrequencyEstimate(frequency={}, filtered={:?})",
            self.frequency, self.filtered
        )
    }
}

/// Analytic-signal result from a Hilbert transform: envelope amplitude,
/// instantaneous phase, and instantaneous frequency.
#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct HilbertResult {
    #[pyo3(get)]
    pub amplitude: f64,
    #[pyo3(get)]
    pub phase: f64,
    #[pyo3(get)]
    pub frequency: Option<f64>,
}

#[pymethods]
impl HilbertResult {
    fn __repr__(&self) -> String {
        format!(
            "HilbertResult(amplitude={}, phase={}, frequency={:?})",
            self.amplitude, self.phase, self.frequency
        )
    }
}

/// Helper to build a [`ComponentVaRResult`] from the portfolio engine.
pub fn make_component_var(cmvar: &mut ComponentMarginalVaR, weights: &[f64]) -> PyResult<Option<ComponentVaRResult>> {
    match cmvar.compute(weights) {
        Some((total, marginal, component)) => Ok(Some(ComponentVaRResult {
            total_var: total,
            marginal: marginal.to_vec(),
            component: component.to_vec(),
        })),
        None => Ok(None),
    }
}
