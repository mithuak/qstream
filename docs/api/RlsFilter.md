# RlsFilter

> RLS adaptive one-step predictor.

| Field | Value |
|---|---|
| Group | Signal · kalman / state-space / adaptive |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import RlsFilter` |

## Signature

```python
RlsFilter(order=4, lam=1.0)
```

## What it computes

Recursive least squares: exact (not stochastic) minimization of the exponentially-weighted squared prediction error with forgetting factor `lambda`.

## Why use it

Track hidden state or predict the next observation from noisy measurements.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `order` | `int` | `4` | Model, filter, or polynomial order. |
| `lam` | `float` | `1.0` | Forgetting parameter for recursive least squares. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

RLS adaptive one-step predictor.

```text
K = P x / (lambda + x^T P x)
e = x_t - w^T x
w <- w + K e
P <- (P - K x^T P) / lambda
```

Recursive least squares: exact (not stochastic) minimization of the
exponentially-weighted squared prediction error with forgetting factor
`lambda`.

## Public members

- `coefficients` — Property
- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
