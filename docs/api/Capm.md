# Capm

> CAPM alpha/beta via rolling excess-return regression.

| Field | Value |
|---|---|
| Group | Finance · factors |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import Capm` |

## Signature

```python
Capm(period)
```

## What it computes

Capital Asset Pricing Model regression; returns `alpha`, `beta`, and `r2`.

## Why use it

Estimate an asset's sensitivity to benchmark or factor returns and attribute performance.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `required` | Rolling length or effective horizon; see the formula for this indicator's convention. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `asset_return` | `float` | Return of the asset being analyzed. |
| `benchmark_return` | `float` | Return of the comparison benchmark. |
| `risk_free` | `float` | Risk-free return for this observation. |

**Output:** `FactorResult | None` — Regression coefficients and fit statistics..

See [FactorResult](/api/FactorResult) for its output fields.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

CAPM alpha/beta via rolling excess-return regression.

```text
r_a - rf = alpha + beta (r_m - rf) + e
beta = Cov(r_a, r_m) / Var(r_m)
```

Capital Asset Pricing Model regression; returns `alpha`, `beta`, and `r2`.

## Public members

- `reset()` — Clear indicator state.
- `update(asset_return, benchmark_return, risk_free=0.0)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
