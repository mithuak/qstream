# Low-Latency Finance + Signal Processing Library Design

## 1. Goal

Build an internal **Python-first quantitative feature library** with **Rust performance and predictable low latency**, covering the finance and finance-relevant signal-processing functions in the companion Excel workbook.

The runtime design intentionally avoids **NumPy, pandas, SciPy, and Numba as required dependencies**. Python is the control/API layer; Rust owns hot-path computation, streaming state, rolling buffers, matrix state, FFT state, and result generation.

Primary goals:

- Very low per-update latency for live market data.
- One implementation for streaming and batch-style iteration.
- Python ergonomics similar to Wickra.
- Rust-owned memory to minimize Python allocations.
- No DataFrame conversion and no mandatory ndarray conversion.
- Support scalar, OHLC, pair, factor, portfolio, spectral, and structured outputs.
- Allow heavier algorithms to update on a configurable cadence rather than on every tick.
- Keep experimental algorithms separable from the stable core.

## 2. Public Python API

### 2.1 Scalar streaming

```python
from qstream import EwmaVolatility

ind = EwmaVolatility(alpha=0.06)

for r in returns:
    value = ind.update(r)
```

### 2.2 OHLC streaming

```python
from qstream import GarmanKlass

ind = GarmanKlass(period=20)
value = ind.update(open, high, low, close)
```

### 2.3 Pair / benchmark streaming

```python
from qstream import RollingBeta

beta = RollingBeta(period=60)
value = beta.update(asset_return, market_return)
```

### 2.4 Factor models

```python
from qstream import FamaFrench3

model = FamaFrench3(period=252)
result = model.update(asset_return, market, smb, hml, risk_free)
print(result.alpha, result.betas, result.r2)
```

### 2.5 Vector-valued results

```python
from qstream import WelchPSD

psd = WelchPSD(window=512, update_every=32)
result = psd.update(price)

if result is not None:
    print(result.frequencies)
    print(result.power)
    print(result.dominant_frequency)
```

The Python API may return Python `float`, `bool`, tuples, lists, or lightweight PyO3 result objects. These objects should wrap Rust-owned or compact copied data. Large Python-object creation should be avoided on every tick.

## 3. Architecture

```text
Python
  |
  | PyO3
  v
Rust extension module
  |
  +-- core/
  |    +-- ring_buffer
  |    +-- moments
  |    +-- covariance
  |    +-- regression
  |    +-- quantiles
  |    +-- matrix
  |
  +-- finance/
  |    +-- volatility
  |    +-- risk
  |    +-- microstructure
  |    +-- factors
  |    +-- portfolio
  |    +-- derivatives
  |
  +-- signal/
       +-- filters
       +-- kalman
       +-- spectral
       +-- wavelet
       +-- decomposition
       +-- regime
       +-- prediction
```

Use `maturin` to build and package the Python extension.

Recommended Rust dependencies:

- `pyo3` — Python bindings.
- `rustfft` — FFT / STFT / Welch / spectral core.
- `num-complex` — complex arithmetic.
- `faer` or `nalgebra` — matrix operations for Kalman, regressions, factor models, and spectral methods.
- `rayon` — optional parallelism for explicitly requested batch/heavy operations.

Avoid a large dependency graph in the latency-sensitive core. Feature-gate heavy modules if necessary.

## 4. No-NumPy Runtime Strategy

The library should not require NumPy.

For streaming:

```python
indicator.update(x)
```

passes Python scalar values directly to PyO3 and immediately into Rust.

For small vector inputs:

```python
portfolio.update([0.01, -0.003, 0.004])
```

PyO3 converts the Python sequence into a temporary Rust slice/vector only when required. For latency-sensitive portfolio paths, expose a fixed-arity or reusable-input-object API to avoid repeated allocations.

For bulk processing, expose:

```python
indicator.update_many(values)
```

accepting Python `list`, `tuple`, `array.array`, or a buffer-protocol object. Internally process the buffer in Rust. A future optional NumPy adapter can be added without making NumPy mandatory.

## 5. Core Rust Traits

Use multiple traits rather than forcing every function into one scalar interface.

```rust
pub trait ScalarIndicator {
    type Output;
    fn update(&mut self, value: f64) -> Option<Self::Output>;
    fn reset(&mut self);
}

pub trait OhlcIndicator {
    type Output;
    fn update(&mut self, open: f64, high: f64, low: f64, close: f64)
        -> Option<Self::Output>;
    fn reset(&mut self);
}

pub trait PairIndicator {
    type Output;
    fn update(&mut self, x: f64, y: f64) -> Option<Self::Output>;
    fn reset(&mut self);
}
```

Factor and portfolio models can use dedicated internal interfaces.

## 6. Shared Primitives

Implement these before individual indicators:

1. `RingBuffer<T>`
2. `RollingSum`
3. `RollingMean`
4. `RollingVariance`
5. `RollingMoments` through fourth moment
6. `RollingCovariance`
7. `EWMA`
8. `OnlineRegression`
9. `RollingRegression`
10. `CovarianceMatrix`
11. `RollingQuantile`
12. `TailAccumulator`
13. `DrawdownState`
14. `ComplexRingBuffer`
15. `FFTPlanCache`
16. `WindowFunctions`
17. `SmallMatrixState`

Most finance functions should become thin wrappers around these primitives.

## 7. Latency Rules

### Tier A — true per-tick streaming

Target extremely low and stable latency.

Examples:

- EWMA volatility
- GARCH(1,1)
- Alpha/beta
- Kalman / alpha-beta tracker
- LMS / RLS
- Page-Hinkley
- Teager-Kaiser energy
- Goertzel
- rolling moments
- drawdown state

Requirements:

- No heap allocation in `update()` after initialization.
- No Python object allocation except the returned scalar.
- O(1) or fixed-size matrix work.
- Preallocate buffers.
- Release the GIL only when work is large enough to justify it.

### Tier B — bounded rolling work

Examples:

- rolling VaR / CVaR
- rolling regression
- Savitzky-Golay
- DWT / MODWT
- periodogram

Requirements:

- Fixed-capacity ring buffers.
- Reuse scratch memory.
- Avoid rebuilding plans or matrices.
- Optional `update_every` parameter.

### Tier C — heavy periodic computation

Examples:

- Welch PSD
- STFT
- multitaper PSD
- MUSIC
- ESPRIT
- VMD
- particle filters
- synchrosqueezing

Expose:

```python
WelchPSD(window=512, update_every=32)
```

Every observation updates the internal ring buffer, but heavy computation runs only every N samples.

## 8. Memory and Allocation Policy

- Allocate rolling buffers in constructors.
- Reuse scratch vectors.
- Cache FFT plans.
- Prefer stack-backed small matrices where dimensions are known.
- Avoid returning large arrays every tick.
- For spectral outputs, return a result only when a computation actually ran.
- Allow callers to query scalar summaries without materializing the full spectrum:

```python
psd.dominant_frequency
psd.spectral_entropy
psd.peak_power
```

## 9. Result Types

Examples:

```text
StateEstimate
- value: float
- velocity: float | None
- variance/covariance

FactorResult
- alpha: float
- betas: list[float]
- r2: float
- residual_variance: float

SpectrumResult
- frequencies: list[float]
- power: list[float]
- dominant_frequency: float | None
- peak_power: float | None

WaveletResult
- coefficients: list[float]
- scales: list[float] | None

ChangeResult
- changed: bool
- score: float
```

Prefer compact result objects and expose individual properties lazily where practical.

## 10. Module Layout

```text
qstream/
├── Cargo.toml
├── pyproject.toml
├── src/
│   ├── lib.rs
│   ├── core/
│   │   ├── ring.rs
│   │   ├── moments.rs
│   │   ├── covariance.rs
│   │   ├── regression.rs
│   │   ├── quantile.rs
│   │   ├── matrix.rs
│   │   └── fft.rs
│   ├── finance/
│   │   ├── volatility/
│   │   ├── risk/
│   │   ├── microstructure/
│   │   ├── factor/
│   │   ├── portfolio/
│   │   └── derivatives/
│   ├── signal/
│   │   ├── filters/
│   │   ├── kalman/
│   │   ├── spectral/
│   │   ├── wavelet/
│   │   ├── regime/
│   │   ├── prediction/
│   │   └── decomposition/
│   └── python/
│       ├── finance.rs
│       ├── signal.rs
│       └── result_types.rs
└── tests/
```

## 11. Implementation Plan

### Phase 1 — foundation

Implement:

- PyO3 + maturin package.
- scalar and OHLC indicator conventions.
- ring buffer.
- rolling moments.
- covariance.
- EWMA.
- regression.
- benchmark harness.
- result/error conventions.

Deliverable: package installable via `pip install ./dist/*.whl`.

### Phase 2 — simple finance

Implement high-value low-complexity functions first:

- Parkinson volatility.
- Garman-Klass.
- Rogers-Satchell.
- EWMA volatility.
- GARCH(1,1).
- tracking error.
- CAPM alpha/beta.
- Jensen alpha.
- capture ratios.
- Cornish-Fisher VaR.
- CDaR.
- Rachev ratio.

### Phase 3 — adaptive/state-space

Implement:

- alpha-beta tracker.
- Kalman.
- adaptive Kalman.
- square-root Kalman.
- EKF.
- UKF.
- LMS.
- RLS.
- Page-Hinkley.
- Teager-Kaiser.
- zero-crossing rate.

### Phase 4 — factor + portfolio

Implement shared matrix/regression state, then:

- FF3.
- FF5.
- Carhart.
- generic N-factor regression.
- Treynor-Mazuy.
- EWMA covariance matrix.
- component/marginal VaR.
- portfolio risk metrics.

### Phase 5 — spectral core

Implement:

- FFT plan cache.
- periodogram.
- Welch PSD.
- STFT.
- Goertzel.
- spectral shape features.
- cross-spectrum.
- coherence.
- AR spectrum.

Do not allocate FFT plans per call.

### Phase 6 — wavelets

Implement:

- DWT.
- MODWT.
- stationary wavelet transform.
- CWT/Morlet.
- wavelet variance.
- wavelet correlation.
- wavelet coherence.
- wavelet packet decomposition.

### Phase 7 — heavy/experimental

Implement last:

- MUSIC.
- ESPRIT.
- Matrix Pencil.
- VMD.
- particle filters.
- synchrosqueezing.
- higher-order spectra.

These should be feature-gated or marked experimental until performance and numerical behavior are validated.

## 12. Testing

For every function:

1. Deterministic reference-value test.
2. Streaming-vs-reference test.
3. Warm-up behavior test.
4. Reset test.
5. NaN/Inf/input validation test.
6. Long-running stability test.
7. Latency benchmark.

Python reference code may use SciPy/pandas/NumPy in development tests if desired, but these must remain development-only dependencies.

## 13. Benchmarking

Measure:

- p50 / p95 / p99 update latency.
- allocations per update.
- memory per indicator instance.
- throughput in updates/sec.
- heavy-function cadence cost.
- Python-to-Rust crossing overhead.

Benchmark separately for:

```text
pure Rust update
Python -> PyO3 -> Rust update
1 indicator
100 indicators
1000 indicators
```

The Python boundary can dominate extremely tiny operations, so also provide optional grouped execution:

```python
engine = FeatureEngine([
    EwmaVolatility(...),
    Garch(...),
    PageHinkley(...),
])

result = engine.update(price, return_)
```

This amortizes PyO3 call overhead when many features are calculated on each tick.

## 14. Recommended Optimization: FeatureEngine

For the lowest latency, add a Rust-owned feature graph:

```python
engine = FeatureEngine(config)

features = engine.update(
    timestamp,
    open,
    high,
    low,
    close,
    volume,
)
```

One Python call can update dozens or hundreds of Rust indicators.

This is likely more important for end-to-end latency than micro-optimizing each individual PyO3 method.

## 15. Definition of Done

A function is complete when:

- Python API is documented in the Excel/API docs.
- No required NumPy/pandas dependency.
- Rust implementation passes reference tests.
- Streaming state is bounded or clearly documented.
- No avoidable allocation in the hot path.
- Benchmark exists.
- Inputs/outputs are stable.
- Heavy algorithms expose cadence controls where appropriate.
