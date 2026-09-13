# JohnsonSUVaR

> Johnson-SU Value-at-Risk.

| Field | Value |
|---|---|
| Group | Experimental · Finance · tail risk |
| Source | `src/python/finance_experimental.rs` |
| Python import | `from qstream import JohnsonSUVaR` |

## Signature

```python
JohnsonSUVaR(window=64, alpha=0.01, update_every=16)
```

## What it computes

Fits a Johnson SU distribution (which can match any skewness/kurtosis combination) and reads the `alpha`-quantile.

## Why use it

Track downside exposure and compare risk across return histories.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `alpha` | `float` | `0.01` | Quantile level used for the Johnson-SU VaR estimate. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Johnson-SU Value-at-Risk.

```text
VaR = -(xi + lambda sinh((z_alpha - gamma)/delta))
```

Fits a Johnson SU distribution (which can match any skewness/kurtosis
combination) and reads the `alpha`-quantile.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
