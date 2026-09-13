# ChirpZTransform

> Chirp-Z Transform (zoom FFT).

| Field | Value |
|---|---|
| Group | Experimental · Time-frequency |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import ChirpZTransform` |

## Signature

```python
ChirpZTransform(window=64, f_start=0.1, f_end=0.3, n_points=64, update_every=16)
```

## What it computes

Evaluates the Z-transform on a spiral contour, allowing high-resolution zoom over a narrow frequency band.

## Why use it

Locate changing frequency content in time.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `f_start` | `float` | `0.1` | Lower endpoint of the zoom-frequency range. |
| `f_end` | `float` | `0.3` | Upper endpoint of the zoom-frequency range. |
| `n_points` | `int` | `64` | Number of output points in the zoom transform. |
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

Chirp-Z Transform (zoom FFT).

```text
X(z_k) = sum_n x[n] z_k^{-n},  z_k = A W^{-k}
```

Evaluates the Z-transform on a spiral contour, allowing high-resolution
zoom over a narrow frequency band.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
