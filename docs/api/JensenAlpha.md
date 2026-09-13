# JensenAlpha

> Jensen's alpha (intercept of excess-return regression).

| Field | Value |
|---|---|
| Group | Finance · factors |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import JensenAlpha` |

## Signature

```python
JensenAlpha(period)
```

## What it computes

Risk-adjusted excess return; the intercept of the CAPM regression.

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

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Jensen's alpha (intercept of excess-return regression).

```text
alpha = E[r_a] - rf - beta (E[r_m] - rf)
```

Risk-adjusted excess return; the intercept of the CAPM regression.

## Public members

- `reset()` — Clear indicator state.
- `update(asset_return, benchmark_return, risk_free=0.0)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
