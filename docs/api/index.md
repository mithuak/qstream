# Python API catalog

This catalog documents **161 canonical public callables** in the installed qstream 0.1.0 build. Signatures and descriptions come from the native Python module; categories follow registration in `src/lib.rs`.

The *Experimental* groups use the project's Phase 7 terminology; these exports are available in the main package without a feature gate.

Select a name for its constructor or function signature, public methods, and source location. Generated pages should be refreshed whenever bindings change.

## Result types

[CaptureResult](/api/CaptureResult) · [ChangeResult](/api/ChangeResult) · [ComponentVaRResult](/api/ComponentVaRResult) · [FactorResult](/api/FactorResult) · [FrequencyEstimate](/api/FrequencyEstimate) · [HilbertResult](/api/HilbertResult) · [PredictionResult](/api/PredictionResult) · [SpectralShapeResult](/api/SpectralShapeResult) · [SpectrumResult](/api/SpectrumResult) · [StateEstimate](/api/StateEstimate) · [TermStructureResult](/api/TermStructureResult) · [WaveletResult](/api/WaveletResult)

## Finance · volatility

[EwmaVariance](/api/EwmaVariance) · [EwmaVolatility](/api/EwmaVolatility) · [Garch](/api/Garch) · [GarmanKlass](/api/GarmanKlass) · [Parkinson](/api/Parkinson) · [RealizedVolatility](/api/RealizedVolatility) · [RogersSatchell](/api/RogersSatchell) · [RollingReturns](/api/RollingReturns) · [RollingVolatility](/api/RollingVolatility)

## Finance · microstructure

[AbdiRanaldo](/api/AbdiRanaldo) · [CorwinSchultz](/api/CorwinSchultz) · [GlostenMilgrom](/api/GlostenMilgrom)

## Finance · tail risk

[ConditionalDrawdownAtRisk](/api/ConditionalDrawdownAtRisk) · [ConditionalValueAtRisk](/api/ConditionalValueAtRisk) · [CornishFisherVaR](/api/CornishFisherVaR) · [RachevRatio](/api/RachevRatio) · [ValueAtRisk](/api/ValueAtRisk)

## Finance · factors

[Capm](/api/Capm) · [Carhart4](/api/Carhart4) · [FamaFrench3](/api/FamaFrench3) · [FamaFrench5](/api/FamaFrench5) · [JensenAlpha](/api/JensenAlpha) · [MultiFactorModel](/api/MultiFactorModel) · [RollingBeta](/api/RollingBeta) · [RollingBetaStability](/api/RollingBetaStability) · [TrackingError](/api/TrackingError) · [TreynorMazuy](/api/TreynorMazuy)

## Finance · performance

[CaptureRatios](/api/CaptureRatios) · [ConditionalSharpe](/api/ConditionalSharpe) · [DeflatedSharpeRatio](/api/DeflatedSharpeRatio) · [GainLossRatio](/api/GainLossRatio) · [LoAutocorrelationSharpe](/api/LoAutocorrelationSharpe) · [OmegaRatio](/api/OmegaRatio) · [SharpeRatio](/api/SharpeRatio) · [SortinoRatio](/api/SortinoRatio)

## Finance · portfolio

[ComponentMarginalVaR](/api/ComponentMarginalVaR) · [EwmaCovarianceMatrix](/api/EwmaCovarianceMatrix) · [PortfolioDuration](/api/PortfolioDuration) · [PortfolioReturns](/api/PortfolioReturns) · [risk_parity_weights](/api/risk_parity_weights)

## Finance · derivatives

[ImpliedVolatility](/api/ImpliedVolatility) · [VarianceSwap](/api/VarianceSwap) · [VixTermStructure](/api/VixTermStructure) · [bs_price](/api/bs_price)

## Signal · filters

[AdaptiveNotchFilter](/api/AdaptiveNotchFilter) · [KolmogorovZurbenko](/api/KolmogorovZurbenko) · [SavitzkyGolay](/api/SavitzkyGolay) · [WienerFilter](/api/WienerFilter)

## Signal · kalman / state-space / adaptive

[AdaptiveKalman](/api/AdaptiveKalman) · [AlphaBetaTracker](/api/AlphaBetaTracker) · [ExtendedKalman](/api/ExtendedKalman) · [KalmanFilter](/api/KalmanFilter) · [LmsFilter](/api/LmsFilter) · [RlsFilter](/api/RlsFilter) · [SquareRootKalman](/api/SquareRootKalman) · [UnscentedKalman](/api/UnscentedKalman)

## Signal · regime / energy

[PageHinkley](/api/PageHinkley) · [TeagerKaiser](/api/TeagerKaiser) · [ZeroCrossingRate](/api/ZeroCrossingRate)

## Signal · spectral

[ArSpectrum](/api/ArSpectrum) · [BartlettMethod](/api/BartlettMethod) · [BlackmanTukey](/api/BlackmanTukey) · [Coherence](/api/Coherence) · [CrossSpectrum](/api/CrossSpectrum) · [FftSpectralDensity](/api/FftSpectralDensity) · [Goertzel](/api/Goertzel) · [MultitaperPsd](/api/MultitaperPsd) · [Periodogram](/api/Periodogram) · [WelchPsd](/api/WelchPsd) · [spectral_shape](/api/spectral_shape)

## Signal · time-frequency (STFT / Hilbert)

[HilbertTransform](/api/HilbertTransform) · [InstantaneousFrequency](/api/InstantaneousFrequency) · [ShortTimeFourierTransform](/api/ShortTimeFourierTransform)

## Signal · wavelets

[CwtMorlet](/api/CwtMorlet) · [DiscreteWaveletTransform](/api/DiscreteWaveletTransform) · [Modwt](/api/Modwt) · [MultiresolutionAnalysis](/api/MultiresolutionAnalysis) · [WaveletCoherence](/api/WaveletCoherence) · [WaveletCorrelation](/api/WaveletCorrelation) · [WaveletPacket](/api/WaveletPacket) · [WaveletVariance](/api/WaveletVariance)

## Signal · prediction

[LatticePredictionErrorFilter](/api/LatticePredictionErrorFilter) · [LpcPredictor](/api/LpcPredictor) · [burg_ar](/api/burg_ar) · [levinson_durbin](/api/levinson_durbin)

## Grouped execution

[FeatureEngine](/api/FeatureEngine)

## Experimental · Result types

[PyBicoherenceResult](/api/PyBicoherenceResult) · [PyBispectrumResult](/api/PyBispectrumResult) · [PyCumulantResult](/api/PyCumulantResult) · [PyDecompositionResult](/api/PyDecompositionResult) · [PyEVTResult](/api/PyEVTResult) · [PyVarianceSwapResult](/api/PyVarianceSwapResult)

## Experimental · Spectral heavy methods

[CaponSpectrum](/api/CaponSpectrum) · [EspritSpectrum](/api/EspritSpectrum) · [MatrixPencilSpectrum](/api/MatrixPencilSpectrum) · [MinimumNormSpectrum](/api/MinimumNormSpectrum) · [MusicSpectrum](/api/MusicSpectrum) · [PisarenkoSpectrum](/api/PisarenkoSpectrum) · [PronySpectrum](/api/PronySpectrum)

## Experimental · Decomposition

[EmdDecomposition](/api/EmdDecomposition) · [LmdDecomposition](/api/LmdDecomposition) · [MatchingPursuitDecomposition](/api/MatchingPursuitDecomposition) · [SynchrosqueezingTransform](/api/SynchrosqueezingTransform) · [VmdDecomposition](/api/VmdDecomposition)

## Experimental · Higher-order spectra

[BicoherenceAnalysis](/api/BicoherenceAnalysis) · [BispectrumAnalysis](/api/BispectrumAnalysis) · [HigherOrderCumulants](/api/HigherOrderCumulants)

## Experimental · Time-frequency

[ChirpZTransform](/api/ChirpZTransform) · [ConstantQTransform](/api/ConstantQTransform) · [FractionalFourierTransform](/api/FractionalFourierTransform) · [KurtogramAnalysis](/api/KurtogramAnalysis) · [ReassignedSpectrogram](/api/ReassignedSpectrogram) · [StockwellTransform](/api/StockwellTransform) · [WignerVilleDistribution](/api/WignerVilleDistribution)

## Experimental · Wavelet extras

[CrossWaveletTransform](/api/CrossWaveletTransform) · [DualTreeCwt](/api/DualTreeCwt) · [EmpiricalWaveletTransform](/api/EmpiricalWaveletTransform) · [StationaryWaveletDenoise](/api/StationaryWaveletDenoise) · [SureShrinkDenoise](/api/SureShrinkDenoise) · [TunableQWavelet](/api/TunableQWavelet) · [WaveletPhaseSynchrony](/api/WaveletPhaseSynchrony) · [WaveletRegression](/api/WaveletRegression)

## Experimental · Heavy Kalman

[CubatureKalman](/api/CubatureKalman) · [EnsembleKalman](/api/EnsembleKalman) · [ParticleFilter](/api/ParticleFilter)

## Experimental · Cycle

[PhaseLockedLoop](/api/PhaseLockedLoop)

## Experimental · FastICA

[FastICA](/api/FastICA)

## Experimental · Finance · tail risk

[EVTGpdTailRisk](/api/EVTGpdTailRisk) · [EntropicVaR](/api/EntropicVaR) · [JohnsonSUVaR](/api/JohnsonSUVaR) · [SpectralRiskMeasure](/api/SpectralRiskMeasure)

## Experimental · Finance · portfolio optimizers

[ExponentiallyWeightedPortfolio](/api/ExponentiallyWeightedPortfolio) · [MaxDiversification](/api/MaxDiversification)

## Experimental · Finance · realized volatility

[RealizedKernel](/api/RealizedKernel) · [TwoScaleRealizedVariance](/api/TwoScaleRealizedVariance)

## Experimental · Finance · volatility derivatives

[DemeterfiVarianceSwap](/api/DemeterfiVarianceSwap) · [FlemingOstdiekWhaleyVIX](/api/FlemingOstdiekWhaleyVIX) · [VandermeerVIX](/api/VandermeerVIX)

## Experimental · Signal · Spectral and filtering

[ApesSpectrum](/api/ApesSpectrum) · [CepstralAnalysis](/api/CepstralAnalysis) · [DaniellPeriodogram](/api/DaniellPeriodogram) · [EigenvectorFrequencyEstimator](/api/EigenvectorFrequencyEstimator) · [ModifiedCovarianceArSpectrum](/api/ModifiedCovarianceArSpectrum) · [MultipleCoherence](/api/MultipleCoherence) · [MultivariateSpectralAnalysis](/api/MultivariateSpectralAnalysis) · [PartialCoherence](/api/PartialCoherence) · [ParzenPeriodogram](/api/ParzenPeriodogram) · [SpectralEnvelope](/api/SpectralEnvelope) · [WienerHopfFilter](/api/WienerHopfFilter)

## Convenience aliases

These names refer to canonical exports and do not have separate implementations.

| Alias | Canonical export |
|---|---|
| `BetaAlpha` | [Capm](/api/Capm) |
| `CVaR` | [ConditionalValueAtRisk](/api/ConditionalValueAtRisk) |
| `CWT` | [CwtMorlet](/api/CwtMorlet) |
| `Carhart` | [Carhart4](/api/Carhart4) |
| `DWT` | [DiscreteWaveletTransform](/api/DiscreteWaveletTransform) |
| `ES` | [ConditionalValueAtRisk](/api/ConditionalValueAtRisk) |
| `ExpectedShortfall` | [ConditionalValueAtRisk](/api/ConditionalValueAtRisk) |
| `FamaFrenchFive` | [FamaFrench5](/api/FamaFrench5) |
| `FamaFrenchThree` | [FamaFrench3](/api/FamaFrench3) |
| `LMS` | [LmsFilter](/api/LmsFilter) |
| `MODWT` | [Modwt](/api/Modwt) |
| `RLS` | [RlsFilter](/api/RlsFilter) |
| `UpDownCapture` | [CaptureRatios](/api/CaptureRatios) |
| `WelchPSD` | [WelchPsd](/api/WelchPsd) |
