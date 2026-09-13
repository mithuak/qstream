# WelchPsd

> Welch overlapped-periodogram PSD.

| Field | Value |
|---|---|
| Group | Signal · spectral |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import WelchPsd` |

## Signature

```python
WelchPsd(window=256, update_every=16, fs=1.0, window_type="hann")
```

## What it computes

Averages K windowed (typically Hann) segments with overlap to reduce variance; `update_every` controls the recompute cadence.

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

Welch overlapped-periodogram PSD.

```text
P(f) = (1/K) * sum_k |X_k(f)|^2 / (fs * sum w^2)
```

Averages K windowed (typically Hann) segments with overlap to reduce variance; `update_every` controls the recompute cadence.

## Example

```python
from qstream import WelchPsd

psd = WelchPsd(window=64, update_every=8)
for sample in [float(i % 10) for i in range(96)]:
    result = psd.update(sample)
    if result is not None:
        print(result.dominant_frequency, result.peak_power)
```

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
