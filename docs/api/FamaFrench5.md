# FamaFrench5

> Fama-French five-factor model (market, SMB, HML, RMW, CMA).

| Field | Value |
|---|---|
| Group | Finance · factors |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import FamaFrench5` |

## Signature

```python
FamaFrench5(period=252)
```

## What it computes

Adds profitability (RMW) and investment (CMA) factors to the three-factor model; returns `FactorResult(alpha, betas, r2, residual_variance)`.

## Why use it

Estimate an asset's sensitivity to benchmark or factor returns and attribute performance.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `252` | Rolling length or effective horizon; see the formula for this indicator's convention. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `asset_return` | `float` | Return of the asset being analyzed. |
| `market` | `float` | Market factor return. |
| `smb` | `float` | Size factor return (small minus big). |
| `hml` | `float` | Value factor return (high minus low). |
| `rmw` | `float` | Profitability factor return (robust minus weak). |
| `cma` | `float` | Investment factor return (conservative minus aggressive). |
| `risk_free` | `float` | Risk-free return for this observation. |

**Output:** `FactorResult` — Regression coefficients and fit statistics..

See [FactorResult](/api/FactorResult) for its output fields.

## Formula and method

Fama-French five-factor model (market, SMB, HML, RMW, CMA).

```text
r - rf = alpha + b1 (rm - rf) + b2 SMB + b3 HML + b4 RMW + b5 CMA + e
```

Adds profitability (RMW) and investment (CMA) factors to the three-factor
model; returns `FactorResult(alpha, betas, r2, residual_variance)`.

## Public members

- `reset()` — Clear indicator state.
- `update(asset_return, market, smb, hml, rmw, cma, risk_free=0.0)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
