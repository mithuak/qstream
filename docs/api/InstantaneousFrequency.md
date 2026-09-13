# InstantaneousFrequency

> Instantaneous frequency via the Hilbert analytic signal.

| Field | Value |
|---|---|
| Group | Signal · time-frequency (STFT / Hilbert) |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import InstantaneousFrequency` |

## Signature

```python
InstantaneousFrequency(window=128, update_every=64, fs=1.0)
```

## What it computes

Convenience alias over [`HilbertTransform`]; returns the frequency at the window center.

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

**Output:** `float | None`.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Instantaneous frequency via the Hilbert analytic signal.

```text
f_t = (1 / 2 pi) * d(arg(x_t + j H[x_t])) / dt
```

Convenience alias over [`HilbertTransform`]; returns the frequency at the
window center.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
