# HilbertTransform

> Hilbert transform (analytic signal).

| Field | Value |
|---|---|
| Group | Signal · time-frequency (STFT / Hilbert) |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import HilbertTransform` |

## Signature

```python
HilbertTransform(window=128, update_every=64, fs=1.0)
```

## What it computes

On each cadence returns the instantaneous amplitude (envelope), phase, and frequency at the window center.

## Why use it

Track how oscillation, envelope, or frequency changes over time.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `128` | Number of recent samples retained for each calculation. |
| `update_every` | `int` | `64` | Number of updates between full recalculations after the window is ready. |
| `fs` | `float` | `1.0` | Sampling frequency in samples per unit time. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `HilbertResult | None` — Instantaneous amplitude, phase, and frequency..

See [HilbertResult](/api/HilbertResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Hilbert transform (analytic signal).

```text
z_t = x_t + j H[x_t]
amplitude = |z_t|
phase = arg(z_t)
frequency = d(phase)/dt
```

On each cadence returns the instantaneous amplitude (envelope), phase, and
frequency at the window center.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
