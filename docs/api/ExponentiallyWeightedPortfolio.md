# ExponentiallyWeightedPortfolio

> Exponentially Weighted Portfolio (EWP).

| Field | Value |
|---|---|
| Group | Experimental · Finance · portfolio optimizers |
| Source | `src/python/finance_experimental.rs` |
| Python import | `from qstream import ExponentiallyWeightedPortfolio` |

## Signature

```python
ExponentiallyWeightedPortfolio(n_assets=3, lambda=0.94)
```

## What it computes

Minimum-variance weights from an exponentially weighted covariance matrix.

## Why use it

Derive portfolio weights using a specified risk or diversification objective.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `n_assets` | `int` | `3` | Number of assets expected in each input vector. |
| `lambda` | `float` | `0.94` | Exponential decay or forgetting parameter. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `returns` | `list[float]` | Return vector for the current observation. |

**Output:** `list[float] | None`.

A `None` element means no value is available yet. This commonly occurs during warmup.

## Formula and method

Exponentially Weighted Portfolio (EWP).

```text
w = Sigma^{-1} 1 / (1^T Sigma^{-1} 1)     (minimum variance)
Sigma_t = lambda Sigma_{t-1} + (1-lambda) r r^T
```

Minimum-variance weights from an exponentially weighted covariance matrix.

## Public members

- `portfolio_volatility(weights)` — Calculate portfolio volatility for the supplied weights.
- `reset()` — Clear indicator state.
- `update(returns)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
