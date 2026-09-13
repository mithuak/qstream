# MaxDiversification

> Maximum Diversification Portfolio.

| Field | Value |
|---|---|
| Group | Experimental · Finance · portfolio optimizers |
| Source | `src/python/finance_experimental.rs` |
| Python import | `from qstream import MaxDiversification` |

## Signature

```python
MaxDiversification(n_assets=3, period=32, update_every=8)
```

## What it computes

Maximizes the diversification ratio `w^T sigma / sqrt(w^T Sigma w)`.

## Why use it

Derive portfolio weights using a specified risk or diversification objective.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `n_assets` | `int` | `3` | Number of assets expected in each input vector. |
| `period` | `int` | `32` | Rolling length or effective horizon; see the formula for this indicator's convention. |
| `update_every` | `int` | `8` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `returns` | `list[float]` | Return vector for the current observation. |

**Output:** `list[float] | None`.

A `None` element means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Maximum Diversification Portfolio.

```text
w = Sigma^{-1} sigma / (1^T Sigma^{-1} sigma)
```

Maximizes the diversification ratio `w^T sigma / sqrt(w^T Sigma w)`.

## Public members

- `reset()` — Clear indicator state.
- `update(returns)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
