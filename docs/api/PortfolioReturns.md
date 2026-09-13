# PortfolioReturns

> Weighted portfolio return stream.

| Field | Value |
|---|---|
| Group | Finance · portfolio |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import PortfolioReturns` |

## Signature

```python
PortfolioReturns(n_assets)
```

## What it computes

Dot product of asset returns and weights; tracks cumulative and mean return.

## Why use it

Combine asset observations into portfolio-level returns or risk measures.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `n_assets` | `int` | `required` | Number of assets expected in each input vector. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `returns` | `list[float]` | Return vector for the current observation. |
| `weights` | `list[float]` | Portfolio weights aligned with the asset vector. |

**Output:** `float`.

## Formula and method

Weighted portfolio return stream.

```text
r_p = sum_i w_i r_i
```

Dot product of asset returns and weights; tracks cumulative and mean
return.

## Public members

- `cumulative` — Property
- `mean` — Property
- `reset()` — Clear indicator state.
- `update(returns, weights)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
