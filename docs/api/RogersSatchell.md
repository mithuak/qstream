# RogersSatchell

> Rogers-Satchell drift-independent OHLC volatility.

| Field | Value |
|---|---|
| Group | Finance · volatility |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import RogersSatchell` |

## Signature

```python
RogersSatchell(period)
```

## What it computes

OHLC estimator that is insensitive to drift (trend).

## Why use it

Monitor changing volatility for risk limits, position sizing, or volatility forecasts.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `required` | Rolling length or effective horizon; see the formula for this indicator's convention. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `open` | `float` | Opening price for the current bar. |
| `high` | `float` | Highest price in the current bar. |
| `low` | `float` | Lowest price in the current bar. |
| `close` | `float` | Closing price for the current bar. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Rogers-Satchell drift-independent OHLC volatility.

```text
sigma^2 = ln(H/C) ln(H/O) + ln(L/C) ln(L/O)
```

OHLC estimator that is insensitive to drift (trend).

## Public members

- `reset()` — Clear indicator state.
- `update(open, high, low, close)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
