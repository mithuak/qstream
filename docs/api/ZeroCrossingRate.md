# ZeroCrossingRate

> Zero-crossing rate over a rolling window.

| Field | Value |
|---|---|
| Group | Signal · regime / energy |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import ZeroCrossingRate` |

## Signature

```python
ZeroCrossingRate(window=32, threshold=0.0)
```

## What it computes

Fraction of samples that cross (or lie within a threshold of) zero; a coarse estimate of the dominant frequency.

## Why use it

Detect changes in the behavior or energy of a stream.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `32` | Number of recent samples retained for each calculation. |
| `threshold` | `float` | `0.0` | Decision threshold for the detector or filter. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Zero-crossing rate over a rolling window.

```text
zcr = (1 / N) * sum_t 1[ x_{t-1} x_t < 0 or |x_t| < threshold ]
```

Fraction of samples that cross (or lie within a threshold of) zero; a
coarse estimate of the dominant frequency.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
