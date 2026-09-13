# RealizedVolatility

> Realized volatility.

| Field | Value |
|---|---|
| Group | Finance · volatility |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import RealizedVolatility` |

## Signature

```python
RealizedVolatility(period)
```

## What it computes

Sum of squared returns over a rolling window (no mean removal).

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

Realized volatility.

```text
RV = sqrt( sum_t r_t^2 )
```

Sum of squared returns over a rolling window (no mean removal).

## Example

```python
from qstream import RealizedVolatility

rv = RealizedVolatility(period=3)
for daily_return in [0.01, -0.005, 0.012, 0.004]:
    value = rv.update(daily_return)
    if value is not None:
        print(value)
```

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
