# GarmanKlass

> Garman-Klass OHLC volatility.

| Field | Value |
|---|---|
| Group | Finance · volatility |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import GarmanKlass` |

## Signature

```python
GarmanKlass(period)
```

## What it computes

OHLC range-based volatility estimator using open, high, low, close.

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

Garman-Klass OHLC volatility.

```text
sigma^2 = 0.5 (ln(H/L))^2 - (2 ln 2 - 1) (ln(C/O))^2
```

OHLC range-based volatility estimator using open, high, low, close.

## Example

```python
from qstream import GarmanKlass

gk = GarmanKlass(period=2)
for bar in [(100.0, 102.0, 99.0, 101.0), (101.0, 103.0, 100.0, 102.0)]:
    print(gk.update(*bar))  # open, high, low, close
```

## Public members

- `reset()` — Clear indicator state.
- `update(open, high, low, close)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
