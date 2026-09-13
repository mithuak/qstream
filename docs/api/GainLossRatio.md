# GainLossRatio

> Gain-loss (Bernardo-Ledoit) ratio.

| Field | Value |
|---|---|
| Group | Finance · performance |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import GainLossRatio` |

## Signature

```python
GainLossRatio(period)
```

## What it computes

Ratio of total positive returns to total negative returns.

## Why use it

Evaluate return quality relative to risk, downside, or a benchmark.

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

Gain-loss (Bernardo-Ledoit) ratio.

```text
GLR = sum( max(0, r) ) / sum( max(0, -r) )
```

Ratio of total positive returns to total negative returns.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
