# DemeterfiVarianceSwap

> Demeterfi Variance Swap Replication.

| Field | Value |
|---|---|
| Group | Experimental · Finance · volatility derivatives |
| Source | `src/python/finance_experimental.rs` |
| Python import | `from qstream import DemeterfiVarianceSwap` |

## Signature

```python
DemeterfiVarianceSwap(window=22, period=0.082, annualization=252.0, update_every=5)
```

## What it computes

Fair variance-swap strike from the model-free replication formula.

## Why use it

Analyze volatility index or variance-swap quantities.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `22` | Number of recent samples retained for each calculation. |
| `period` | `float` | `0.082` | Rolling length or effective horizon; see the formula for this indicator's convention. |
| `annualization` | `float` | `252.0` | Scale factor used to annualize the result. |
| `update_every` | `int` | `5` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `PyVarianceSwapResult | None`.

See [PyVarianceSwapResult](/api/PyVarianceSwapResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Demeterfi Variance Swap Replication.

```text
K_var = E[RV] + convexity correction
```

Fair variance-swap strike from the model-free replication formula.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
