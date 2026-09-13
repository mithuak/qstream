# SortinoRatio

> Sortino ratio with a minimum acceptable return (MAR).

| Field | Value |
|---|---|
| Group | Finance · performance |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import SortinoRatio` |

## Signature

```python
SortinoRatio(period=252, mar=0.0)
```

## What it computes

Excess return per unit of downside deviation (punishes only losses).

## Why use it

Evaluate return quality relative to risk, downside, or a benchmark.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `252` | Rolling length or effective horizon; see the formula for this indicator's convention. |
| `mar` | `float` | `0.0` | Minimum acceptable return for the downside comparison. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Sortino ratio with a minimum acceptable return (MAR).

```text
Sortino = (mean(r) - MAR) / sqrt( mean( min(0, r - MAR)^2 ) )
```

Excess return per unit of downside deviation (punishes only losses).

## Public members

- `downside_deviation` — Property
- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
