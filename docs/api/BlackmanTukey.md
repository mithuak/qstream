# BlackmanTukey

> Blackman-Tukey spectral estimation (FFT of lag-windowed autocorrelation).

| Field | Value |
|---|---|
| Group | Signal · spectral |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import BlackmanTukey` |

## Signature

```python
BlackmanTukey(window=128, max_lag=32, update_every=32)
```

## What it computes

Fourier transform of the lag-windowed autocorrelation `r_h` with a triangular (Bartlett) lag window `w_h`.

## Why use it

Find periodic components or track how signal power is distributed by frequency.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `128` | Number of recent samples retained for each calculation. |
| `max_lag` | `int` | `32` | Largest autocovariance lag included in the estimate. |
| `update_every` | `int` | `32` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `SpectrumResult | None` — Frequency bins, power values, dominant frequency, and peak power..

See [SpectrumResult](/api/SpectrumResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Blackman-Tukey spectral estimation (FFT of lag-windowed autocorrelation).

```text
P(f) = sum_{h=-M}^{M} w_h r_h e^{-j 2 pi f h}
```

Fourier transform of the lag-windowed autocorrelation `r_h` with a
triangular (Bartlett) lag window `w_h`.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
