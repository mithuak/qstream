# AbdiRanaldo

> Abdi-Ranaldo closing-price spread estimator.

| Field | Value |
|---|---|
| Group | Finance · microstructure |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import AbdiRanaldo` |

## Signature

```python
AbdiRanaldo(period=20)
```

## What it computes

Bid-ask spread from the covariance of close and mid-price using only daily OHLC data.

## Why use it

Estimate trading frictions, spread, or price impact from market observations.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `20` | Rolling length or effective horizon; see the formula for this indicator's convention. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `high` | `float` | Highest price in the current bar. |
| `low` | `float` | Lowest price in the current bar. |
| `close` | `float` | Closing price for the current bar. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Abdi-Ranaldo closing-price spread estimator.

```text
S = 2 sqrt( mean( (ln(C_t/M_t))^2 ) )
M_t = (H_t + L_t) / 2
```

Bid-ask spread from the covariance of close and mid-price using only
daily OHLC data.

## Public members

- `reset()` — Clear indicator state.
- `update(high, low, close)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
