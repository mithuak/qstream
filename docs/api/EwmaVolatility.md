# EwmaVolatility

> RiskMetrics-style EWMA volatility.

| Field | Value |
|---|---|
| Group | Finance · volatility |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import EwmaVolatility` |

## Signature

```python
EwmaVolatility(alpha=0.06)
```

## What it computes

`alpha` is the weight on the new squared return; the RiskMetrics daily default is `alpha = 0.06`.

## Why use it

Monitor changing volatility for risk limits, position sizing, or volatility forecasts.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `alpha` | `float` | `0.06` | Weight on the newest squared return (`lambda = 1 - alpha`). |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

RiskMetrics-style EWMA volatility.

```text
sigma_t^2 = lambda sigma_{t-1}^2 + (1 - lambda) r_t^2
lambda = 1 - alpha
```

`alpha` is the weight on the new squared return; the RiskMetrics daily
default is `alpha = 0.06`.

## Example

```python
from qstream import EwmaVolatility

vol = EwmaVolatility(alpha=0.06)
for daily_return in [0.01, -0.005, 0.012]:
    print(vol.update(daily_return))
```

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).
- `variance` — Property

[Back to the API catalog](/api/).
