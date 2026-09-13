# KolmogorovZurbenko

> Kolmogorov-Zurbenko filter (cascaded moving averages).

| Field | Value |
|---|---|
| Group | Signal · filters |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import KolmogorovZurbenko` |

## Signature

```python
KolmogorovZurbenko(window=11, passes=3)
```

## What it computes

Repeated moving-average smoothing removes high-frequency noise while preserving the low-frequency trend with a sharp transition band.

## Why use it

Smooth or adapt a noisy series before measurement or prediction.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `11` | Number of recent samples retained for each calculation. |
| `passes` | `int` | `3` | Number of repeated smoothing passes. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Kolmogorov-Zurbenko filter (cascaded moving averages).

```text
y_t = MA_w^p (x_t)     # p passes of a w-point moving average
```

Repeated moving-average smoothing removes high-frequency noise while
preserving the low-frequency trend with a sharp transition band.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
