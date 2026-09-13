# SavitzkyGolay

> Savitzky-Golay polynomial smoothing over a rolling window.

| Field | Value |
|---|---|
| Group | Signal · filters |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import SavitzkyGolay` |

## Signature

```python
SavitzkyGolay(window=9, order=2, deriv=0)
```

## What it computes

Fits a low-order polynomial in a sliding odd window and evaluates it (or its `deriv`-th derivative) at the center.

## Why use it

Smooth or adapt a noisy series before measurement or prediction.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `9` | Number of recent samples retained for each calculation. |
| `order` | `int` | `2` | Model, filter, or polynomial order. |
| `deriv` | `int` | `0` | Order of the derivative to estimate; zero requests smoothing. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Savitzky-Golay polynomial smoothing over a rolling window.

```text
y_t = sum_{k=-m}^{m} c_k x_{t+k}
c = (A^T A)^{-1} A^T   (least-squares fit of a polynomial of order p)
```

Fits a low-order polynomial in a sliding odd window and evaluates it (or
its `deriv`-th derivative) at the center.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
