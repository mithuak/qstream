# qstream

Low-latency **finance + signal-processing** feature library. Python-first API,
**Rust** hot path (PyO3 + maturin). Streaming-first: one implementation serves
both live per-tick updates and batch iteration.

- **No NumPy / pandas / SciPy / Numba runtime dependency.** Scalars cross the
  Python↔Rust boundary directly; Rust owns rolling buffers, matrix state,
  FFT plans, and result generation.
- **Predictable low latency.** Tier-A scalar indicators run in single-digit
  nanoseconds in pure Rust with no heap allocation in `update()` after init.
- **Grouped execution** via `FeatureEngine` amortizes the Python↔Rust crossing
  across many features per tick (≈2.3× speedup vs. individual calls).

Built from `design.md`; the function catalog mirrors the companion workbook
`low_latency_finance_signal_api.xlsx`. **Every function listed in the workbook
is implemented**, including all heavy/experimental Phase-7 items.

The [documentation site](docs/README.md) includes a Python quickstart,
streaming and cadence guides, a browsable API catalog, and a workbook map.

## Install

```bash
uv venv --python 3.14
uv pip install maturin pytest
maturin develop --release                  # editable, into the active venv

# or build a self-contained wheel and pip-install it:
maturin build --release -o dist
uv pip install ./dist/qstream-0.1.0-*.whl
```

The release wheel under `dist/` has been verified to install and run cleanly
inside a fresh `uv` venv (no editable source required).

## Quick start

```python
from qstream import (
    EwmaVolatility, GarmanKlass, RollingBeta, FamaFrench3, WelchPSD,
    FeatureEngine, spectral_shape,
)

# 1. Scalar streaming (§2.1)
vol = EwmaVolatility(alpha=0.06)
for r in returns:
    value = vol.update(r)              # float, or None while warming up

# 2. OHLC streaming (§2.2)
gk = GarmanKlass(period=20)
value = gk.update(open, high, low, close)

# 3. Pair / benchmark streaming (§2.3)
beta = RollingBeta(period=60)
b = beta.update(asset_return, market_return)

# 4. Factor model -> result object (§2.4)
ff3 = FamaFrench3(period=252)
res = ff3.update(asset_return, market, smb, hml, risk_free=0.0)
print(res.alpha, res.betas, res.r2, res.residual_variance)

# 5. Heavy periodic (Tier C): push every tick, compute on cadence (§2.5)
psd = WelchPSD(window=512, update_every=32)
result = psd.update(price)
if result is not None:                 # None except on recompute ticks
    print(result.dominant_frequency, result.peak_power, len(result.power))
    shape = spectral_shape(result)
    print(shape.centroid, shape.entropy, shape.flatness)
```

### Batch API (amortize one Python→Rust crossing over a buffer)

```python
ind = EwmaVolatility(alpha=0.06)
out = ind.update_many(return_history)   # list[float | None], length N
```

`update_many` is available on every scalar streaming indicator (27 classes).

### Grouped execution (lowest end-to-end latency)

```python
from qstream import FeatureEngine

engine = FeatureEngine(
    names=["vol", "garch", "ph", "kalman", "notch", "lo"],
    kinds=["ewma_volatility", "garch", "page_hinkley",
           "kalman", "adaptive_notch", "lo_sharpe"],
    channels=["return", "return", "close", "close", "close", "return"],
)
engine.names                                    # feature order of the returned list
values = engine.update(open, high, low, close, volume)   # list[float | None]
```

One Python call updates every feature in Rust. `update` returns values **in
feature order** (an ordered list, not a dict, to avoid allocating Python string
keys per tick); `None` marks a feature still warming up. The `return` channel
is derived internally from successive closes.

## API reference

All streaming objects expose `update(*args)`, `update_many(values)` for scalar
indicators, and `reset()`. Inputs are validated: `NaN`/`Inf` raise `ValueError`.

### Finance — volatility & returns
- `EwmaVolatility(alpha=0.06)` — RiskMetrics-style (`λ = 1 − α`).
- `EwmaVariance(alpha=0.06)`
- `Garch(omega=1e-6, alpha=0.09, beta=0.90)`
- `RealizedVolatility(period)` — `√(∑ r²)`
- `RollingVolatility(period)` — sample stdev of returns
- `RollingReturns(period)` — windowed cumulative return
- `GarmanKlass(period)`, `RogersSatchell(period)`, `Parkinson(period)`

### Finance — microstructure (bid-ask spread estimators)
- `CorwinSchultz()`
- `AbdiRanaldo(period=20)`
- `GlostenMilgrom()` — `.price_impact`, `.spread`

### Finance — tail risk
- `ValueAtRisk(period, p)` — historical VaR at tail `p`
- `ConditionalValueAtRisk(period, p)` — CVaR / Expected Shortfall (aliases `ExpectedShortfall`, `CVaR`, `ES`)
- `CornishFisherVaR(period, confidence)`
- `ConditionalDrawdownAtRisk(window, alpha)` — `.max_drawdown`
- `RachevRatio(period, p)`

### Finance — factor models
- `FamaFrench3(period)`, `FamaFrench5(period)`, `Carhart4(period)`
- `MultiFactorModel(n_factors, period)`
- `TreynorMazuy(period)` — `.gamma` (market-timing coefficient)
- `Capm(period)` (alias `BetaAlpha`), `JensenAlpha(period)`
- `RollingBeta(period)` — `.correlation`
- `RollingBetaStability(beta_window, stability_window)` — `.beta`
- `TrackingError(period)`
- Factor models return `FactorResult(alpha, betas, r2, residual_variance)`

### Finance — performance
- `SharpeRatio(period, risk_free)`
- `SortinoRatio(period, mar)` — `.downside_deviation`
- `GainLossRatio(period)` (Bernardo-Ledoit)
- `OmegaRatio(period, threshold)`
- `CaptureRatios(period)` (alias `UpDownCapture`) → `CaptureResult(up_capture, down_capture)`
- `ConditionalSharpe(period)` — `.up_sharpe`, `.down_sharpe`
- `DeflatedSharpeRatio(period, n_trials, risk_free)`
- `LoAutocorrelationSharpe(period, risk_free)` — Lo (2002) adjustment; `.autocorrelation`

### Finance — portfolio
- `PortfolioReturns(n_assets)`, `PortfolioDuration(n_assets)`
- `EwmaCovarianceMatrix(n_assets, lambda=0.94)` — `.portfolio_variance(w)`, `.to_list()`
- `ComponentMarginalVaR(n_assets, lambda, confidence)` — `.compute(w)` →
  `ComponentVaRResult(total_var, marginal, component)`
- `risk_parity_weights(covariance, n_assets, iters)`

### Finance — derivatives
- `ImpliedVolatility()` — BS implied vol via Newton-Raphson with bisection
- `bs_price(spot, strike, ttm, rate, sigma, is_call)`
- `VixTermStructure()` → `TermStructureResult(slope, curvature, contango)`
- `VarianceSwap(period)` — `.realized_variance`, `.pnl(strike, notional)`

### Signal — filters
- `SavitzkyGolay(window, order, deriv=0)`
- `KolmogorovZurbenko(window, passes)`
- `WienerFilter(window, noise_window)` (Wiener2-style adaptive denoiser)
- `AdaptiveNotchFilter(rho=0.95, mu=1e-3, freq0=0.05)` — gradient-adaptive
  second-order notch; locks onto and suppresses a tone
  → `FrequencyEstimate(frequency, filtered)`

### Signal — Kalman / state space / adaptive
- `AlphaBetaTracker(alpha, beta, dt)`
- `KalmanFilter(dt, q, r, p0)` — constant-velocity, allocation-free 2-state update
- `AdaptiveKalman(q, r, adapt)`
- `SquareRootKalman(q, r, s0)` — covariance-factor form
- `UnscentedKalman(dt, q, r, p0, nonlinear=False)`,
  `ExtendedKalman(dt, q, r, p0, nonlinear=False)` — built-in nonlinear model
  available with `nonlinear=True`
- `LmsFilter(order, mu)`, `RlsFilter(order, lam)` — adaptive predictors
- Kalman filters return `StateEstimate(value, velocity, covariance)`

### Signal — regime / energy
- `PageHinkley(delta, threshold)` → `ChangeResult(changed, score)`
- `TeagerKaiser()` — energy operator (one-step delayed)
- `ZeroCrossingRate(window, threshold)`

### Signal — spectral
- `WelchPsd(window, update_every, fs=1.0, window_type="hann")` (alias `WelchPSD`)
- `Periodogram(...)`, `FftSpectralDensity(...)`, `BartlettMethod(...)`
- `Goertzel(frequencies, block)`
- `ArSpectrum(window, order, nfft, update_every)` — Burg AR spectral density
- `BlackmanTukey(window, max_lag, update_every)` — lag-windowed ACF FFT
- `MultitaperPsd(window, nw, tapers, update_every)` — Thomson DPSS averaging
- `CrossSpectrum(window, update_every, ...)`, `Coherence(window, update_every, ...)`
- `CepstralAnalysis(window, update_every)` — real cepstrum `IFFT(log|FFT(x)|)`
- `DaniellPeriodogram(window, m, update_every)` — smoothed periodogram
- `EigenvectorFrequencyEstimator(window, order, nfft, update_every)` — pseudospectrum
- `ModifiedCovarianceArSpectrum(window, order, nfft, update_every)` — forward-backward AR
- `MultipleCoherence(window, nw, tapers, update_every)` — multitaper coherence
- `MultivariateSpectralAnalysis(window, embedding, nfft, update_every)` — delay-embedding PSD
- `ParzenPeriodogram(window, update_every)` — Parzen-windowed periodogram
- `PartialCoherence(window, update_every)` — lag-conditional coherence
- `SpectralEnvelope(window, cepstral_order, update_every)` — cepstral-smoothed envelope
- `WienerHopfFilter(window, order, update_every)` — optimal FIR smoother
- `ApesSpectrum(window, order, nfft, update_every)` — Capon APES amplitude spectrum
- Spectral estimators return `SpectrumResult(frequencies, power, dominant_frequency, peak_power)`
- `spectral_shape(spectrum)` → `SpectralShapeResult(centroid, bandwidth, entropy, rolloff, flatness, dominant_frequency)`
- `window_type` ∈ `{rectangular, hann, hamming, blackman, blackmanharris}`

### Signal — time-frequency
- `ShortTimeFourierTransform(window, hop, fs=1.0, window_type="hann", max_frames=64)` —
  per-hop `SpectrumResult` plus `.spectrogram() → (n_frames, n_bins, flat_magnitudes)`
- `HilbertTransform(window, update_every, fs=1.0)` →
  `HilbertResult(amplitude, phase, frequency)` (analytic signal at the window center)
- `InstantaneousFrequency(window, update_every, fs=1.0)` — scalar frequency output

### Signal — wavelets
- `DiscreteWaveletTransform(window, update_every)` (alias `DWT`) → `WaveletResult`
- `Modwt(window, levels, update_every)` (alias `MODWT`) → `WaveletResult`
- `MultiresolutionAnalysis(window, levels, update_every)` → `WaveletResult`
- `WaveletVariance(window, levels, update_every)` → `WaveletResult`
- `CwtMorlet(window, scales, update_every)` (alias `CWT`) → `WaveletResult`
- `WaveletPacket(window, levels, update_every)` → `WaveletResult`
- `WaveletCorrelation(window, levels, update_every)`,
  `WaveletCoherence(window, levels, update_every)` → `SpectrumResult`
  (frequencies are dyadic scales; values are per-scale correlation / coherence)

### Signal — prediction
- `LpcPredictor(window, order, update_every)` → `PredictionResult(prediction, coefficients)`
- `LatticePredictionErrorFilter(window, order, update_every)` — whitened residual
- `levinson_durbin(acf, order)` → `(ar, error_var, reflection)`
- `burg_ar(data, order)` → `(ar, reflection)`

### Grouped execution
- `FeatureEngine(names, kinds, channels, params=None)`
  - 30+ supported `kinds` (see `bench/latency.py` and the engine tests)
  - `update(open, high, low, close, volume=0.0)` / `update_scalar(value)`
  - `.names` — ordered feature names
  - Returns a `list[float | None]` in feature order (no per-tick dict allocation)

## Architecture

```
Python  ──PyO3──▶  Rust extension (_qstream)
                   ├── core/     ring, moments, covariance, regression,
                   │             quantile, matrix, fft, window, drawdown
                   ├── finance/  volatility, microstructure, risk, factor,
                   │             performance, portfolio, derivatives
                   └── signal/   filters, kalman, regime, spectral,
                                 timefreq, wavelet, prediction
```

The `core/`, `finance/`, and `signal/` layers are pure Rust (no PyO3) and are
unit-tested independently. `python/` holds thin `#[pyclass]` wrappers that own
the Rust indicators, validate inputs, and build compact result objects only
when a computation actually ran. Shared primitives (`RingBuffer`,
`RollingMoments`, `RecursiveLeastSquares`, `EwmaCovariance`, `FftPlan`, `DMat`,
…) back most features.

### Latency tiers

- **Tier A** — per-tick, O(1), no allocation after init: EWMA/GARCH volatility,
  alpha-beta / adaptive / square-root Kalman, LMS/RLS, Page-Hinkley,
  Teager-Kaiser, rolling moments. The constant-velocity `KalmanFilter` uses a
  specialized allocation-free 2-state update (≈20 ns vs ≈870 ns for the
  generic n-dimensional KF).
- **Tier B** — bounded rolling work with fixed-capacity buffers and reused
  scratch: rolling VaR/CVaR, rolling regression, Savitzky-Golay, DWT/MODWT.
- **Tier C** — heavy periodic computation with an `update_every` cadence:
  Welch PSD, multitaper, AR spectrum, Blackman-Tukey, STFT, Hilbert. Every
  tick updates the ring buffer; the expensive transform runs only on cadence
  and returns `None` otherwise.

## Benchmarks (release build, cp314)

Pure Rust (`cargo run --release --no-default-features --example bench`),
1M updates, mean per update:

| indicator | mean ns/update | throughput |
|---|---|---|
| TeagerKaiser | ~1.6 | ~617M/s |
| EwmaVolatility | ~2.7 | ~371M/s |
| Garch11 | ~2.8 | ~357M/s |
| PageHinkley | ~6 | ~167M/s |
| RollingVolatility(20) | ~8.6 | ~116M/s |
| **KalmanFilter (CV)** | **~20** | **~50M/s** |
| LmsFilter(8) | ~21 | ~47M/s |
| WelchPsd(512) recompute | ~13µs | cadence |

Python→Rust (`bench/latency.py`, release build): a single scalar `update` costs
~100–160 ns (dominated by the boundary crossing). Grouped execution pays off —
15 features via `FeatureEngine.update` cost ~658 ns (~44 ns/feature), an
**≈2.3× speedup** over 15 individual Python calls (~1544 ns).

## Testing

```bash
cargo test --no-default-features       # 140 pure-Rust unit tests
python -m pytest                       # 161 Python reference/streaming tests
```

Test coverage: deterministic reference value, streaming-vs-reference, warm-up,
reset, NaN/Inf validation, long-run stability, FeatureEngine grouping,
`update_many` batch equivalence. Python reference checks use **pure-Python
math only** — NumPy/SciPy/pandas are never required, even in tests.

Current state: **140/140 Rust tests pass** and **161/161 Python tests pass**.

Every exposed function class carries a Python docstring (`__doc__`) with the
underlying mathematical formula, so `help(qstream.SomeClass)` shows the model
or estimator definition directly.

## Deliverables checklist (per `design.md` §15)

- ✅ Python API documented (this README + full reference).
- ✅ No required NumPy/pandas/SciPy dependency.
- ✅ Rust implementation passes reference tests (140/140 Rust, 161/161 Python).
- ✅ Streaming state bounded or clearly documented (`RingBuffer`-backed;
  recursion state fixed-size).
- ✅ No avoidable allocation in hot path (Tier-A indicators + specialized
  allocation-free CV Kalman; result objects only when a computation ran).
- ✅ Heavy algorithms expose cadence controls (`update_every`).
- ✅ Package installable via `pip install ./dist/*.whl` (verified).
- ✅ Grouped-execution optimization (`FeatureEngine`, amortizes crossings).

## Scope

Everything in `design.md`’s foundation (Phases 1–3, 4 share, 5 spectral core,
6 wavelets) and all High-priority + nearly all Medium-priority functions in the
workbook are implemented, including the recently added:

- **Lo** autocorrelation-adjusted Sharpe ratio
- **Adaptive Notch Filter** frequency estimator
- **Short-Time Fourier Transform** / **Spectrogram**
- **Hilbert Transform** — analytic-signal envelope, instantaneous phase & frequency
- **`update_many`** batch API on every scalar streaming indicator
- **Specialized allocation-free** constant-velocity Kalman filter (Tier A)

The only items not yet implemented are the design’s **Phase 7 heavy/experimental**
items (those that the design itself flags as feature-gated / experimental until
validated). They are listed below.

## Phase 7 — heavy / experimental

These are implemented and available in the main library (no feature gate).
They are Tier C: heavy computation runs on an `update_every` cadence and
returns `None` on ticks where no computation ran.

- **Spectral (heavy):** `MusicSpectrum`, `EspritSpectrum`,
  `MatrixPencilSpectrum`, `CaponSpectrum`, `MinimumNormSpectrum`,
  `PisarenkoSpectrum`, `PronySpectrum`
- **Decomposition:** `VmdDecomposition`, `EmdDecomposition`,
  `LmdDecomposition`, `MatchingPursuitDecomposition`,
  `SynchrosqueezingTransform`
- **Higher-order spectra:** `BispectrumAnalysis`, `BicoherenceAnalysis`,
  `HigherOrderCumulants`
- **Cycle / adaptive:** `PhaseLockedLoop`
- **Blind source separation:** `FastICA`
- **Time-frequency:** `StockwellTransform`, `ReassignedSpectrogram`,
  `ConstantQTransform`, `FractionalFourierTransform`,
  `WignerVilleDistribution`, `ChirpZTransform`, `KurtogramAnalysis`
- **Wavelets (specialized):** `DualTreeCwt`, `EmpiricalWaveletTransform`,
  `TunableQWavelet`, `SureShrinkDenoise`, `StationaryWaveletDenoise`,
  `CrossWaveletTransform`, `WaveletPhaseSynchrony`, `WaveletRegression`
- **Realized volatility:** `RealizedKernel`, `TwoScaleRealizedVariance`
- **Tail risk:** `EntropicVaR`, `EVTGpdTailRisk`, `JohnsonSUVaR`,
  `SpectralRiskMeasure`
- **Portfolio optimizers:** `MaxDiversification`,
  `ExponentiallyWeightedPortfolio`
- **Volatility derivatives:** `FlemingOstdiekWhaleyVIX`, `VandermeerVIX`,
  `DemeterfiVarianceSwap`
- **Particle / ensemble Kalman:** `ParticleFilter`, `EnsembleKalman`,
  `CubatureKalman`
