# FamaFrench3

> Fama-French three-factor model (market, SMB, HML).

| Field | Value |
|---|---|
| Group | Finance · factors |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import FamaFrench3` |

## Signature

```python
FamaFrench3(period=252)
```

## What it computes

Online regression of excess asset return on market, size (SMB), and value (HML) factors; returns `FactorResult(alpha, betas, r2, residual_variance)`.

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
| `risk_free` | `float` | Risk-free return for this observation. |

**Output:** `FactorResult` — Regression coefficients and fit statistics..

See [FactorResult](/api/FactorResult) for its output fields.

## Formula and method

Fama-French three-factor model (market, SMB, HML).

```text
r - rf = alpha + b1 (rm - rf) + b2 SMB + b3 HML + e
```

Online regression of excess asset return on market, size (SMB), and value
(HML) factors; returns `FactorResult(alpha, betas, r2, residual_variance)`.

## Example

```python
from qstream import FamaFrench3

model = FamaFrench3(period=252)
result = model.update(0.01, 0.008, 0.002, -0.001, risk_free=0.0)
print(result.alpha, result.betas, result.r2)
```

## Public members

- `reset()` — Clear indicator state.
- `update(asset_return, market, smb, hml, risk_free=0.0)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
