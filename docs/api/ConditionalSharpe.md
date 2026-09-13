# ConditionalSharpe

> Conditional Sharpe ratio (asset Sharpe conditioned on benchmark direction).

| Field | Value |
|---|---|
| Group | Finance · performance |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import ConditionalSharpe` |

## Signature

```python
ConditionalSharpe(period)
```

## What it computes

Performance split by benchmark up/down regime.

## Why use it

Evaluate return quality relative to risk, downside, or a benchmark.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `required` | Rolling length or effective horizon; see the formula for this indicator's convention. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `asset_return` | `float` | Return of the asset being analyzed. |
| `benchmark_return` | `float` | Return of the comparison benchmark. |

**Output:** `None`.

The update changes internal state. Read a public property or calculation method for the current result.

## Formula and method

Conditional Sharpe ratio (asset Sharpe conditioned on benchmark direction).

```text
up_sharpe = Sharpe(r_a | r_b > 0)
down_sharpe = Sharpe(r_a | r_b < 0)
```

Performance split by benchmark up/down regime.

## Public members

- `down_sharpe` — Property
- `is_ready` — Property
- `reset()` — Clear indicator state.
- `up_sharpe` — Property
- `update(asset_return, benchmark_return)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
