# Architecture

The public package is Python-first, with PyO3 wrappers around a Rust core.
The core owns numerical state and has no runtime dependency on NumPy, pandas,
SciPy, or Numba.

```text
Python qstream API
        │ PyO3 validation and result conversion
        ▼
Rust core
  ├─ core/      ring buffers, moments, covariance, regression, FFT, matrices
  ├─ finance/   volatility, risk, factors, performance, portfolio, derivatives
  └─ signal/    filters, state space, spectral, wavelet, prediction
```

The design separates work by latency cost:

| Tier | Typical work | Output pattern |
|---|---|---|
| A | Constant-time recursive updates, such as EWMA, GARCH, or Page-Hinkley | Each update after warmup |
| B | Bounded rolling-window work, such as VaR, regression, or filtering | Each update after warmup |
| C | Spectral, time-frequency, wavelet, or other heavy computation | When its cadence is due |

The Python wrappers own Rust indicators, validate inputs, and convert emitted
results. The lower Rust layers can be tested independently with
`cargo test --no-default-features`; Python integration tests run with
`python -m pytest` after installing the extension.

`FeatureEngine` groups supported scalar features to amortize the Python/Rust
boundary. It is an optimization for feature sets that can be expressed as
named kinds over OHLCV or derived-return channels. Standalone classes remain
the interface for other input shapes and structured results.

The project design is maintained in the repository-root `design.md`; this site
describes current usage. Read that file directly when working in a checkout.
