# ComponentMarginalVaR

> Component & marginal VaR from a streaming EWMA covariance.

| Field | Value |
|---|---|
| Group | Finance · portfolio |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import ComponentMarginalVaR` |

## Signature

```python
ComponentMarginalVaR(n_assets, lambda=0.94, confidence=0.95)
```

## What it computes

Decomposes total VaR into per-asset marginal and component contributions.

## Why use it

Combine asset observations into portfolio-level returns or risk measures.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `n_assets` | `int` | `required` | Number of assets expected in each input vector. |
| `lambda` | `float` | `0.94` | Exponential decay or forgetting parameter. |
| `confidence` | `float` | `0.95` | Confidence level for the reported risk estimate. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `returns` | `list[float]` | Return vector for the current observation. |

**Output:** `None`.

The update changes internal state. Read a public property or calculation method for the current result.

## Formula and method

Component & marginal VaR from a streaming EWMA covariance.

```text
VaR = z * sqrt( w^T Sigma w )
marginal_i = z * (Sigma w)_i / sqrt( w^T Sigma w )
component_i = w_i * marginal_i
```

Decomposes total VaR into per-asset marginal and component contributions.

## Public members

- `compute(weights)` — Calculate a result from the current state and supplied arguments.
- `reset()` — Clear indicator state.
- `update(returns)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
