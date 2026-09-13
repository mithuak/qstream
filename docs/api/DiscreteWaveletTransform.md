# DiscreteWaveletTransform

> Haar discrete wavelet transform over a rolling power-of-two window.

| Field | Value |
|---|---|
| Group | Signal · wavelets |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import DiscreteWaveletTransform` |

## Signature

```python
DiscreteWaveletTransform(window=64, update_every=16)
```

## What it computes

Decimated two-channel filter bank; returns approximation `a` and detail `d` coefficients at dyadic scales.

## Why use it

Inspect signal behavior at more than one time scale.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
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

Haar discrete wavelet transform over a rolling power-of-two window.

```text
a_j[k] = (a_{j-1}[2k] + a_{j-1}[2k+1]) / sqrt(2)
d_j[k] = (a_{j-1}[2k] - a_{j-1}[2k+1]) / sqrt(2)
```

Decimated two-channel filter bank; returns approximation `a` and detail `d`
coefficients at dyadic scales.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
