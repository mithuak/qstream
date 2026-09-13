# EVTGpdTailRisk

> EVT Peaks-Over-Threshold / GPD tail risk.

| Field | Value |
|---|---|
| Group | Experimental · Finance · tail risk |
| Source | `src/python/finance_experimental.rs` |
| Python import | `from qstream import EVTGpdTailRisk` |

## Signature

```python
EVTGpdTailRisk(window=128, alpha=0.01, threshold_quantile=0.95, update_every=32)
```

## What it computes

Fits a Generalized Pareto Distribution to exceedances over a high threshold and computes VaR / expected shortfall.

## Why use it

Track downside exposure and compare risk across return histories.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `128` | Number of recent samples retained for each calculation. |
| `alpha` | `float` | `0.01` | Tail level used in the GPD risk estimate. |
| `threshold_quantile` | `float` | `0.95` | Quantile above which observations enter the extreme-value tail fit. |
| `update_every` | `int` | `32` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `PyEVTResult | None`.

See [PyEVTResult](/api/PyEVTResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

EVT Peaks-Over-Threshold / GPD tail risk.

```text
VaR = u + (beta/xi) * ((N/Nu * (1-alpha))^{-xi} - 1)
ES  = (VaR + beta - xi u) / (1 - xi)
```

Fits a Generalized Pareto Distribution to exceedances over a high
threshold and computes VaR / expected shortfall.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
