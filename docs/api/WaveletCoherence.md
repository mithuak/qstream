# WaveletCoherence

> Wavelet coherence per scale.

| Field | Value |
|---|---|
| Group | Signal · wavelets |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import WaveletCoherence` |

## Signature

```python
WaveletCoherence(window=64, levels=3, update_every=16)
```

## What it computes

Per-scale magnitude-squared coherence in [0, 1].

## Why use it

Inspect signal behavior at more than one time scale.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `levels` | `int` | `3` | Number of wavelet decomposition scales. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `x` | `float` | Current observation from the first series. |
| `y` | `float` | Current observation from the second series. |

**Output:** `SpectrumResult | None` — Frequency bins, power values, dominant frequency, and peak power..

See [SpectrumResult](/api/SpectrumResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Wavelet coherence per scale.

```text
C_j = |sum W_x_j * W_y_j*|^2 / (sum |W_x_j|^2 * sum |W_y_j|^2)
```

Per-scale magnitude-squared coherence in [0, 1].

## Public members

- `reset()` — Clear indicator state.
- `update(x, y)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
