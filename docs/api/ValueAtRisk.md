# ValueAtRisk

> Historical Value-at-Risk over a rolling window (positive loss magnitude).

| Field | Value |
|---|---|
| Group | Finance · tail risk |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import ValueAtRisk` |

## Signature

```python
ValueAtRisk(period=252, p=0.05)
```

## What it computes

The loss that is exceeded with probability `p` (the `p`-th empirical quantile of the return distribution, negated).

## Why use it

Track downside exposure and compare risk across return histories.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `252` | Rolling length or effective horizon; see the formula for this indicator's convention. |
| `p` | `float` | `0.05` | Tail probability or probability threshold; see the indicator formula. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Historical Value-at-Risk over a rolling window (positive loss magnitude).

```text
VaR_p = -quantile_p(returns)
```

The loss that is exceeded with probability `p` (the `p`-th empirical
quantile of the return distribution, negated).

## Example

```python
from qstream import ValueAtRisk

var = ValueAtRisk(period=3, p=0.05)
for daily_return in [0.01, -0.02, 0.005, -0.01]:
    print(var.update(daily_return))
```

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
