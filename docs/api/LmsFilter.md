# LmsFilter

> LMS adaptive one-step predictor.

| Field | Value |
|---|---|
| Group | Signal · kalman / state-space / adaptive |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import LmsFilter` |

## Signature

```python
LmsFilter(order=4, mu=0.01)
```

## What it computes

Least-mean-squares adaptive FIR filter; cheap stochastic gradient descent on the mean-squared prediction error.

## Why use it

Track hidden state or predict the next observation from noisy measurements.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `order` | `int` | `4` | Model, filter, or polynomial order. |
| `mu` | `float` | `0.01` | Adaptation step size for the filter. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

LMS adaptive one-step predictor.

```text
y_t = w^T x
e_t = x_t - y_t
w <- w + 2 mu e_t x
```

Least-mean-squares adaptive FIR filter; cheap stochastic gradient descent
on the mean-squared prediction error.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
