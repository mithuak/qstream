# Coherence

> Magnitude-squared coherence.

| Field | Value |
|---|---|
| Group | Signal · spectral |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import Coherence` |

## Signature

```python
Coherence(window=256, update_every=16, fs=1.0, window_type="hann")
```

## What it computes

Values in [0, 1]; 1 means perfect linear relationship at frequency f.

## Why use it

Find periodic components or track how signal power is distributed by frequency.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `256` | Number of recent samples retained for each calculation. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |
| `fs` | `float` | `1.0` | Sampling frequency in samples per unit time. |
| `window_type` | `str` | `"hann"` | Taper applied to the signal window. |

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

Magnitude-squared coherence.

```text
gamma^2(f) = |S_xy(f)|^2 / (S_xx(f) * S_yy(f))
```

Values in [0, 1]; 1 means perfect linear relationship at frequency f.

## Public members

- `reset()` — Clear indicator state.
- `update(x, y)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
