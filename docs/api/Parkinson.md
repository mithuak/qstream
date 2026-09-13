# Parkinson

> Parkinson high-low range volatility.

| Field | Value |
|---|---|
| Group | Finance · volatility |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import Parkinson` |

## Signature

```python
Parkinson(period)
```

## What it computes

Range-based volatility estimator using only high and low prices.

## Why use it

Monitor changing volatility for risk limits, position sizing, or volatility forecasts.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `required` | Rolling length or effective horizon; see the formula for this indicator's convention. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `high` | `float` | Highest price in the current bar. |
| `low` | `float` | Lowest price in the current bar. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Parkinson high-low range volatility.

```text
sigma^2 = (1 / (4 ln 2)) * mean( (ln(H_t / L_t))^2 )
```

Range-based volatility estimator using only high and low prices.

## Public members

- `reset()` — Clear indicator state.
- `update(high, low)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
