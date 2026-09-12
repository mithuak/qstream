//! qstream: low-latency finance + signal-processing feature library.
//!
//! Python is the control/API layer; Rust owns hot-path computation, streaming
//! state, and result generation. No NumPy/pandas/SciPy runtime dependency.

pub mod core;
pub mod finance;
pub mod python;
pub mod signal;

use pyo3::prelude::*;

use python::engine::FeatureEngine;
use python::experimental as exp;
use python::finance as fin;
use python::finance_experimental as fin_exp;
use python::result_types as rt;
use python::signal as sig;
use python::spectral_missing as sm;

/// Library version (kept in sync with Cargo.toml / pyproject.toml).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[pymodule]
fn _qstream(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", VERSION)?;

    // Result types.
    m.add_class::<rt::StateEstimate>()?;
    m.add_class::<rt::FactorResult>()?;
    m.add_class::<rt::SpectrumResult>()?;
    m.add_class::<rt::WaveletResult>()?;
    m.add_class::<rt::ChangeResult>()?;
    m.add_class::<rt::PredictionResult>()?;
    m.add_class::<rt::TermStructureResult>()?;
    m.add_class::<rt::SpectralShapeResult>()?;
    m.add_class::<rt::ComponentVaRResult>()?;
    m.add_class::<rt::CaptureResult>()?;
    m.add_class::<rt::FrequencyEstimate>()?;
    m.add_class::<rt::HilbertResult>()?;

    // Finance: volatility.
    m.add_class::<fin::EwmaVolatility>()?;
    m.add_class::<fin::EwmaVariance>()?;
    m.add_class::<fin::Garch>()?;
    m.add_class::<fin::RealizedVolatility>()?;
    m.add_class::<fin::RollingVolatility>()?;
    m.add_class::<fin::RollingReturns>()?;
    m.add_class::<fin::GarmanKlass>()?;
    m.add_class::<fin::RogersSatchell>()?;
    m.add_class::<fin::Parkinson>()?;

    // Finance: microstructure.
    m.add_class::<fin::CorwinSchultz>()?;
    m.add_class::<fin::AbdiRanaldo>()?;
    m.add_class::<fin::GlostenMilgrom>()?;

    // Finance: tail risk.
    m.add_class::<fin::ValueAtRisk>()?;
    m.add_class::<fin::ConditionalValueAtRisk>()?;
    m.add_class::<fin::CornishFisherVaR>()?;
    m.add_class::<fin::ConditionalDrawdownAtRisk>()?;
    m.add_class::<fin::RachevRatio>()?;

    // Finance: factors.
    m.add_class::<fin::FamaFrench3>()?;
    m.add_class::<fin::FamaFrench5>()?;
    m.add_class::<fin::Carhart4>()?;
    m.add_class::<fin::MultiFactorModel>()?;
    m.add_class::<fin::TreynorMazuy>()?;
    m.add_class::<fin::Capm>()?;
    m.add_class::<fin::JensenAlpha>()?;
    m.add_class::<fin::RollingBeta>()?;
    m.add_class::<fin::RollingBetaStability>()?;
    m.add_class::<fin::TrackingError>()?;

    // Finance: performance.
    m.add_class::<fin::SharpeRatio>()?;
    m.add_class::<fin::SortinoRatio>()?;
    m.add_class::<fin::GainLossRatio>()?;
    m.add_class::<fin::OmegaRatio>()?;
    m.add_class::<fin::CaptureRatios>()?;
    m.add_class::<fin::ConditionalSharpe>()?;
    m.add_class::<fin::DeflatedSharpeRatio>()?;
    m.add_class::<fin::LoAutocorrelationSharpe>()?;

    // Finance: portfolio.
    m.add_class::<fin::PortfolioReturns>()?;
    m.add_class::<fin::PortfolioDuration>()?;
    m.add_class::<fin::EwmaCovarianceMatrix>()?;
    m.add_class::<fin::ComponentMarginalVaR>()?;
    m.add_function(wrap_pyfunction!(fin::risk_parity_weights, m)?)?;

    // Finance: derivatives.
    m.add_class::<fin::ImpliedVolatility>()?;
    m.add_class::<fin::VixTermStructure>()?;
    m.add_class::<fin::VarianceSwap>()?;
    m.add_function(wrap_pyfunction!(fin::bs_price, m)?)?;

    // Signal: filters.
    m.add_class::<sig::SavitzkyGolay>()?;
    m.add_class::<sig::KolmogorovZurbenko>()?;
    m.add_class::<sig::WienerFilter>()?;
    m.add_class::<sig::AdaptiveNotchFilter>()?;

    // Signal: kalman / state-space / adaptive.
    m.add_class::<sig::AlphaBetaTracker>()?;
    m.add_class::<sig::KalmanFilter>()?;
    m.add_class::<sig::AdaptiveKalman>()?;
    m.add_class::<sig::SquareRootKalman>()?;
    m.add_class::<sig::UnscentedKalman>()?;
    m.add_class::<sig::ExtendedKalman>()?;
    m.add_class::<sig::LmsFilter>()?;
    m.add_class::<sig::RlsFilter>()?;

    // Signal: regime / energy.
    m.add_class::<sig::PageHinkley>()?;
    m.add_class::<sig::TeagerKaiser>()?;
    m.add_class::<sig::ZeroCrossingRate>()?;

    // Signal: spectral.
    m.add_class::<sig::WelchPsd>()?;
    m.add_class::<sig::Periodogram>()?;
    m.add_class::<sig::FftSpectralDensity>()?;
    m.add_class::<sig::BartlettMethod>()?;
    m.add_class::<sig::Goertzel>()?;
    m.add_class::<sig::ArSpectrum>()?;
    m.add_class::<sig::BlackmanTukey>()?;
    m.add_class::<sig::MultitaperPsd>()?;
    m.add_class::<sig::CrossSpectrum>()?;
    m.add_class::<sig::Coherence>()?;
    m.add_function(wrap_pyfunction!(sig::spectral_shape, m)?)?;

    // Signal: time-frequency (STFT / Hilbert).
    m.add_class::<sig::ShortTimeFourierTransform>()?;
    m.add_class::<sig::HilbertTransform>()?;
    m.add_class::<sig::InstantaneousFrequency>()?;

    // Signal: wavelets.
    m.add_class::<sig::DiscreteWaveletTransform>()?;
    m.add_class::<sig::Modwt>()?;
    m.add_class::<sig::MultiresolutionAnalysis>()?;
    m.add_class::<sig::WaveletVariance>()?;
    m.add_class::<sig::CwtMorlet>()?;
    m.add_class::<sig::WaveletPacket>()?;
    m.add_class::<sig::WaveletCorrelation>()?;
    m.add_class::<sig::WaveletCoherence>()?;

    // Signal: prediction.
    m.add_class::<sig::LpcPredictor>()?;
    m.add_class::<sig::LatticePredictionErrorFilter>()?;
    m.add_function(wrap_pyfunction!(sig::levinson_durbin, m)?)?;
    m.add_function(wrap_pyfunction!(sig::burg_ar, m)?)?;

    // Grouped execution.
    m.add_class::<FeatureEngine>()?;

    // ========================================================================
    // Phase 7 experimental features
    // ========================================================================

    // Result types
    m.add_class::<exp::PyDecompositionResult>()?;
    m.add_class::<exp::PyBispectrumResult>()?;
    m.add_class::<exp::PyBicoherenceResult>()?;
    m.add_class::<exp::PyCumulantResult>()?;
    m.add_class::<fin_exp::PyEVTResult>()?;
    m.add_class::<fin_exp::PyVarianceSwapResult>()?;

    // Spectral heavy methods
    m.add_class::<exp::MusicSpectrum>()?;
    m.add_class::<exp::EspritSpectrum>()?;
    m.add_class::<exp::MatrixPencilSpectrum>()?;
    m.add_class::<exp::CaponSpectrum>()?;
    m.add_class::<exp::PisarenkoSpectrum>()?;
    m.add_class::<exp::MinimumNormSpectrum>()?;
    m.add_class::<exp::PronySpectrum>()?;

    // Decomposition
    m.add_class::<exp::VmdDecomposition>()?;
    m.add_class::<exp::EmdDecomposition>()?;
    m.add_class::<exp::LmdDecomposition>()?;
    m.add_class::<exp::MatchingPursuitDecomposition>()?;
    m.add_class::<exp::SynchrosqueezingTransform>()?;

    // Higher-order spectra
    m.add_class::<exp::BispectrumAnalysis>()?;
    m.add_class::<exp::BicoherenceAnalysis>()?;
    m.add_class::<exp::HigherOrderCumulants>()?;

    // Time-frequency
    m.add_class::<exp::StockwellTransform>()?;
    m.add_class::<exp::ReassignedSpectrogram>()?;
    m.add_class::<exp::ConstantQTransform>()?;
    m.add_class::<exp::FractionalFourierTransform>()?;
    m.add_class::<exp::WignerVilleDistribution>()?;
    m.add_class::<exp::ChirpZTransform>()?;
    m.add_class::<exp::KurtogramAnalysis>()?;

    // Wavelet extras
    m.add_class::<exp::DualTreeCwt>()?;
    m.add_class::<exp::EmpiricalWaveletTransform>()?;
    m.add_class::<exp::TunableQWavelet>()?;
    m.add_class::<exp::SureShrinkDenoise>()?;
    m.add_class::<exp::StationaryWaveletDenoise>()?;
    m.add_class::<exp::CrossWaveletTransform>()?;
    m.add_class::<exp::WaveletPhaseSynchrony>()?;
    m.add_class::<exp::WaveletRegression>()?;

    // Heavy Kalman
    m.add_class::<exp::ParticleFilter>()?;
    m.add_class::<exp::EnsembleKalman>()?;
    m.add_class::<exp::CubatureKalman>()?;

    // Cycle
    m.add_class::<exp::PhaseLockedLoop>()?;

    // FastICA
    m.add_class::<exp::FastICA>()?;

    // Finance: tail risk
    m.add_class::<fin_exp::EntropicVaR>()?;
    m.add_class::<fin_exp::EVTGpdTailRisk>()?;
    m.add_class::<fin_exp::JohnsonSUVaR>()?;
    m.add_class::<fin_exp::SpectralRiskMeasure>()?;

    // Finance: portfolio optimizers
    m.add_class::<fin_exp::MaxDiversification>()?;
    m.add_class::<fin_exp::ExponentiallyWeightedPortfolio>()?;

    // Finance: realized volatility
    m.add_class::<fin_exp::RealizedKernel>()?;
    m.add_class::<fin_exp::TwoScaleRealizedVariance>()?;

    // Finance: volatility derivatives
    m.add_class::<fin_exp::FlemingOstdiekWhaleyVIX>()?;
    m.add_class::<fin_exp::VandermeerVIX>()?;
    m.add_class::<fin_exp::DemeterfiVarianceSwap>()?;

    // Remaining spectral / filtering estimators (workbook completion).
    m.add_class::<sm::CepstralAnalysis>()?;
    m.add_class::<sm::DaniellPeriodogram>()?;
    m.add_class::<sm::EigenvectorFrequencyEstimator>()?;
    m.add_class::<sm::ModifiedCovarianceArSpectrum>()?;
    m.add_class::<sm::MultipleCoherence>()?;
    m.add_class::<sm::MultivariateSpectralAnalysis>()?;
    m.add_class::<sm::ParzenPeriodogram>()?;
    m.add_class::<sm::PartialCoherence>()?;
    m.add_class::<sm::SpectralEnvelope>()?;
    m.add_class::<sm::WienerHopfFilter>()?;
    m.add_class::<sm::ApesSpectrum>()?;

    Ok(())
}
