# SharpeRatio

> Rolling Sharpe ratio.

| Field | Value |
|---|---|
| Group | Finance · performance |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import SharpeRatio` |

## Signature

```python
SharpeRatio(period=252, risk_free=0.0)
```

## What it computes

Excess return per unit of total risk over a rolling window.

## Why use it

Evaluate return quality relative to risk, downside, or a benchmark.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `252` | Rolling length or effective horizon; see the formula for this indicator's convention. |
| `risk_free` | `float` | `0.0` | Risk-free return used as the performance baseline. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Rolling Sharpe ratio.

```text
Sharpe = (mean(r) - rf) / std(r)
```

Excess return per unit of total risk over a rolling window.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
