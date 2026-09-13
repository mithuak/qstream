# SynchrosqueezingTransform

> Synchrosqueezed STFT.

| Field | Value |
|---|---|
| Group | Experimental · Decomposition |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import SynchrosqueezingTransform` |

## Signature

```python
SynchrosqueezingTransform(window=64, nfft=128, update_every=16)
```

## What it computes

Reassigns STFT energy to the instantaneous-frequency estimate, sharpening the time-frequency representation.

## Why use it

Separate a composite signal into simpler oscillatory or sparse components.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `nfft` | `int` | `128` | FFT length or spectral grid size. |
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

Synchrosqueezed STFT.

```text
S(f) = sum_t delta(f - omega(t)) |X(t, omega(t))|^2
```

Reassigns STFT energy to the instantaneous-frequency estimate, sharpening
the time-frequency representation.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
