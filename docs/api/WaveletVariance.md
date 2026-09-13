# WaveletVariance

> Wavelet variance per scale (MODWT).

| Field | Value |
|---|---|
| Group | Signal · wavelets |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import WaveletVariance` |

## Signature

```python
WaveletVariance(window=64, levels=..., update_every=16)
```

## What it computes

Variance of MODWT coefficients, partitioning total variance across scales.

## Why use it

Inspect signal behavior at more than one time scale.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `levels` | `int` | `...` | Number of wavelet decomposition scales. |
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

Wavelet variance per scale (MODWT).

```text
sigma^2_j = Var(W_j)
```

Variance of MODWT coefficients, partitioning total variance across scales.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
