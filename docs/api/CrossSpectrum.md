# CrossSpectrum

> Cross-spectral density magnitude.

| Field | Value |
|---|---|
| Group | Signal · spectral |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import CrossSpectrum` |

## Signature

```python
CrossSpectrum(window=256, update_every=16, fs=1.0, window_type="hann")
```

## What it computes

Welch-averaged cross-spectrum between two streams.

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

Cross-spectral density magnitude.

```text
|S_xy(f)| = |E[X(f) Y*(f)]| / (fs * sum w^2)
```

Welch-averaged cross-spectrum between two streams.

## Public members

- `reset()` — Clear indicator state.
- `update(x, y)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
