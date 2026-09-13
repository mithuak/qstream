# Carhart4

> Carhart four-factor model (market, SMB, HML, momentum).

| Field | Value |
|---|---|
| Group | Finance · factors |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import Carhart4` |

## Signature

```python
Carhart4(period=252)
```

## What it computes

Adds the momentum factor (MOM) to the three-factor model; returns `FactorResult(alpha, betas, r2, residual_variance)`.

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
| `mom` | `float` | Momentum factor return. |
| `risk_free` | `float` | Risk-free return for this observation. |

**Output:** `FactorResult` — Regression coefficients and fit statistics..

See [FactorResult](/api/FactorResult) for its output fields.

## Formula and method

Carhart four-factor model (market, SMB, HML, momentum).

```text
r - rf = alpha + b1 (rm - rf) + b2 SMB + b3 HML + b4 MOM + e
```

Adds the momentum factor (MOM) to the three-factor model; returns
`FactorResult(alpha, betas, r2, residual_variance)`.

## Public members

- `reset()` — Clear indicator state.
- `update(asset_return, market, smb, hml, mom, risk_free=0.0)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
