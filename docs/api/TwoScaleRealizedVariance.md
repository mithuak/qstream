# TwoScaleRealizedVariance

> Two-Scale Realized Variance (TSRV).

| Field | Value |
|---|---|
| Group | Experimental · Finance · realized volatility |
| Source | `src/python/finance_experimental.rs` |
| Python import | `from qstream import TwoScaleRealizedVariance` |

## Signature

```python
TwoScaleRealizedVariance(window=64, n_subsamples=5, annualization=252.0, update_every=16)
```

## What it computes

Zhang-Mykland-Ait-Sahalia estimator combining a sparse and an averaged realized variance to cancel microstructure noise.

## Why use it

Estimate variation in high-frequency returns while accounting for sampling effects.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `n_subsamples` | `int` | `5` | Number of interleaved subsamples in the two-scale estimate. |
| `annualization` | `float` | `252.0` | Scale factor used to annualize the result. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Two-Scale Realized Variance (TSRV).

```text
TSRV = RV_avg - (n/K) RV_sparse
```

Zhang-Mykland-Ait-Sahalia estimator combining a sparse and an averaged
realized variance to cancel microstructure noise.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
