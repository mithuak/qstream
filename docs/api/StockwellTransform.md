# StockwellTransform

> Stockwell S-transform.

| Field | Value |
|---|---|
| Group | Experimental · Time-frequency |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import StockwellTransform` |

## Signature

```python
StockwellTransform(window=64, nfft=64, update_every=16)
```

## What it computes

Hybrid STFT/wavelet with a frequency-dependent Gaussian window.

## Why use it

Locate changing frequency content in time.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `nfft` | `int` | `64` | FFT length or spectral grid size. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `SpectrumResult | None` — Frequency bins, power values, dominant frequency, and peak power..

See [SpectrumResult](/api/SpectrumResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Stockwell S-transform.

```text
S(tau, f) = int x(t) (|f|/sqrt(2pi)) e^{-(tau-t)^2 f^2 / 2} e^{-j 2pi f t} dt
```

Hybrid STFT/wavelet with a frequency-dependent Gaussian window.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
