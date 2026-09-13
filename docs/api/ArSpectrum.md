# ArSpectrum

> Autoregressive (Burg) spectral density.

| Field | Value |
|---|---|
| Group | Signal · spectral |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import ArSpectrum` |

## Signature

```python
ArSpectrum(window=128, order=16, nfft=128, update_every=32)
```

## What it computes

AR(p) spectrum from Burg reflection coefficients and the prediction-error variance `sigma^2`.

## Why use it

Find periodic components or track how signal power is distributed by frequency.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `128` | Number of recent samples retained for each calculation. |
| `order` | `int` | `16` | Model, filter, or polynomial order. |
| `nfft` | `int` | `128` | FFT length or spectral grid size. |
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

Autoregressive (Burg) spectral density.

```text
P(f) = sigma^2 / |1 + sum_{i=1}^{p} a_i e^{-j 2 pi f i}|^2
```

AR(p) spectrum from Burg reflection coefficients and the prediction-error
variance `sigma^2`.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
