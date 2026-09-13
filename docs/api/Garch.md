# Garch

> GARCH(1,1) conditional volatility.

| Field | Value |
|---|---|
| Group | Finance · volatility |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import Garch` |

## Signature

```python
Garch(omega=1e-6, alpha=0.09, beta=0.90)
```

## What it computes

Bollerslev GARCH(1,1): conditional variance from past shocks and past variance; the classic volatility-clustering model.

## Why use it

Monitor changing volatility for risk limits, position sizing, or volatility forecasts.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `omega` | `float` | `1e-6` | Baseline conditional-variance term. |
| `alpha` | `float` | `0.09` | Weight on the previous squared return shock. |
| `beta` | `float` | `0.90` | Weight on the previous conditional variance. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

GARCH(1,1) conditional volatility.

```text
sigma_t^2 = omega + alpha r_{t-1}^2 + beta sigma_{t-1}^2
```

Bollerslev GARCH(1,1): conditional variance from past shocks and past
variance; the classic volatility-clustering model.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).
- `variance` — Property

[Back to the API catalog](/api/).
