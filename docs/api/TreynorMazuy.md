# TreynorMazuy

> Treynor-Mazuy market-timing model.

| Field | Value |
|---|---|
| Group | Finance · factors |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import TreynorMazuy` |

## Signature

```python
TreynorMazuy(period=252)
```

## What it computes

Quadratic market-timing regression; `gamma > 0` indicates positive timing ability.

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
| `risk_free` | `float` | Risk-free return for this observation. |

**Output:** `FactorResult` — Regression coefficients and fit statistics..

See [FactorResult](/api/FactorResult) for its output fields.

## Formula and method

Treynor-Mazuy market-timing model.

```text
r - rf = alpha + beta (rm - rf) + gamma (rm - rf)^2 + e
```

Quadratic market-timing regression; `gamma > 0` indicates positive timing
ability.

## Public members

- `gamma` — Property
- `reset()` — Clear indicator state.
- `update(asset_return, market, risk_free=0.0)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
