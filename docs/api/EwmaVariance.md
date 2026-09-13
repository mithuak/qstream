# EwmaVariance

> EWMA of squared returns (variance), returned as volatility.

| Field | Value |
|---|---|
| Group | Finance · volatility |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import EwmaVariance` |

## Signature

```python
EwmaVariance(alpha=0.06)
```

## What it computes

Exponentially-weighted moving average of squared returns.

## Why use it

Monitor changing volatility for risk limits, position sizing, or volatility forecasts.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `alpha` | `float` | `0.06` | Weight on the previous variance in the recurrence. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

EWMA of squared returns (variance), returned as volatility.

```text
v_t = (1 - alpha) r_t^2 + alpha v_{t-1}
output = sqrt(v_t)
```

Exponentially-weighted moving average of squared returns.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
