# MultiFactorModel

> Generic N-factor online regression.

| Field | Value |
|---|---|
| Group | Finance · factors |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import MultiFactorModel` |

## Signature

```python
MultiFactorModel(n_factors, period=252)
```

## What it computes

Online least-squares regression of excess return on `N` supplied factors; returns `FactorResult(alpha, betas, r2, residual_variance)`.

## Why use it

Estimate an asset's sensitivity to benchmark or factor returns and attribute performance.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `n_factors` | `int` | `required` | Number of explanatory factor streams. |
| `period` | `int` | `252` | Rolling length or effective horizon; see the formula for this indicator's convention. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `asset_return` | `float` | Return of the asset being analyzed. |
| `factors` | `list[float]` | Factor-return vector for this observation. |
| `risk_free` | `float` | Risk-free return for this observation. |

**Output:** `FactorResult` — Regression coefficients and fit statistics..

See [FactorResult](/api/FactorResult) for its output fields.

## Formula and method

Generic N-factor online regression.

```text
r - rf = alpha + sum_{i=1}^{N} beta_i F_i + e
```

Online least-squares regression of excess return on `N` supplied factors;
returns `FactorResult(alpha, betas, r2, residual_variance)`.

## Public members

- `reset()` — Clear indicator state.
- `update(asset_return, factors, risk_free=0.0)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
