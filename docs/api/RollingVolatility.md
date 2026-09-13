# RollingVolatility

> Rolling volatility.

| Field | Value |
|---|---|
| Group | Finance · volatility |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import RollingVolatility` |

## Signature

```python
RollingVolatility(period)
```

## What it computes

Sample standard deviation of returns over a rolling window.

## Why use it

Monitor changing volatility for risk limits, position sizing, or volatility forecasts.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `required` | Rolling length or effective horizon; see the formula for this indicator's convention. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Rolling volatility.

```text
sigma = sqrt( sum_t (r_t - mean)^2 / (N-1) )
```

Sample standard deviation of returns over a rolling window.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
