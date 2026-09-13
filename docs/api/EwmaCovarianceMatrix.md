# EwmaCovarianceMatrix

> EWMA multivariate covariance matrix.

| Field | Value |
|---|---|
| Group | Finance · portfolio |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import EwmaCovarianceMatrix` |

## Signature

```python
EwmaCovarianceMatrix(n_assets, lambda=0.94)
```

## What it computes

RiskMetrics exponentially-weighted covariance matrix of `n_assets` returns.

## Why use it

Combine asset observations into portfolio-level returns or risk measures.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `n_assets` | `int` | `required` | Number of assets expected in each input vector. |
| `lambda` | `float` | `0.94` | Exponential decay or forgetting parameter. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `returns` | `list[float]` | Return vector for the current observation. |

**Output:** `None`.

The update changes internal state. Read a public property or calculation method for the current result.

## Formula and method

EWMA multivariate covariance matrix.

```text
Sigma_t = lambda Sigma_{t-1} + (1 - lambda) r_t r_t^T
```

RiskMetrics exponentially-weighted covariance matrix of `n_assets` returns.

## Public members

- `dim` — Property
- `is_ready` — Property
- `portfolio_variance(weights)` — Portfolio variance for a weight vector.
- `reset()` — Clear indicator state.
- `to_list()` — Full covariance as a flat row-major list.
- `update(returns)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
