# StationaryWaveletDenoise

> Stationary (Undecimated) Wavelet Denoising.

| Field | Value |
|---|---|
| Group | Experimental · Wavelet extras |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import StationaryWaveletDenoise` |

## Signature

```python
StationaryWaveletDenoise(window=64, levels=3, update_every=16)
```

## What it computes

Shift-invariant `a trous` wavelet transform with thresholding of detail coefficients.

## Why use it

Analyze or denoise structure across time scales.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `levels` | `int` | `3` | Number of wavelet decomposition scales. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `WaveletResult | None` — Wavelet coefficients and scale information..

See [WaveletResult](/api/WaveletResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Stationary (Undecimated) Wavelet Denoising.

```text
a_{j+1}[t] = (a_j[t] + a_j[t + 2^j]) / 2
d_{j+1}[t] = (a_j[t] - a_j[t + 2^j]) / 2   (a trous)
```

Shift-invariant `a trous` wavelet transform with thresholding of detail
coefficients.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
