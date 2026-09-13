# risk_parity_weights

> Equal-risk-contribution (risk parity) weights from a covariance matrix.

| Field | Value |
|---|---|
| Group | Finance · portfolio |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import risk_parity_weights` |

## Signature

```python
risk_parity_weights(covariance, n_assets, iters=200)
```

## What it computes

Iteratively reweights so every asset contributes equal risk.

## Why use it

Combine asset observations into portfolio-level returns or risk measures.

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `covariance` | `list[float]` | Covariance matrix used to calculate portfolio weights. |
| `n_assets` | `int` | Number of assets represented by the covariance matrix. |
| `iters` | `int` | Maximum iterations of the risk-parity weight solver. |

**Output:** `list[float]`.

## Formula and method

Equal-risk-contribution (risk parity) weights from a covariance matrix.

```text
w_i * (Sigma w)_i = w_j * (Sigma w)_j   for all i, j
```

Iteratively reweights so every asset contributes equal risk.

[Back to the API catalog](/api/).
