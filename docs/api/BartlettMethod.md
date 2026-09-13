# BartlettMethod

> Bartlett averaged-segment periodogram.

| Field | Value |
|---|---|
| Group | Signal · spectral |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import BartlettMethod` |

## Signature

```python
BartlettMethod(window=256, update_every=16, fs=1.0, window_type="hann")
```

## What it computes

Averages non-overlapping segments, reducing variance at the cost of resolution.

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
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `SpectrumResult | None` — Frequency bins, power values, dominant frequency, and peak power..

See [SpectrumResult](/api/SpectrumResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Bartlett averaged-segment periodogram.

```text
P(f) = (1/K) * sum_k |X_k(f)|^2 / (fs * sum w^2)
```

Averages non-overlapping segments, reducing variance at the cost of resolution.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
