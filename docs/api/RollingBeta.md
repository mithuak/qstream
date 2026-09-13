# RollingBeta

> Rolling beta of an asset versus a benchmark.

| Field | Value |
|---|---|
| Group | Finance · factors |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import RollingBeta` |

## Signature

```python
RollingBeta(period)
```

## What it computes

Windowed market sensitivity (systematic risk) of an asset.

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
| `market_return` | `float` | Return of the market benchmark. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Rolling beta of an asset versus a benchmark.

```text
beta = Cov(r_a, r_m) / Var(r_m)
```

Windowed market sensitivity (systematic risk) of an asset.

## Example

```python
from qstream import RollingBeta

beta = RollingBeta(period=3)
for asset_return, market_return in [(0.01, 0.008), (-0.005, -0.004), (0.012, 0.009)]:
    print(beta.update(asset_return, market_return))
```

## Public members

- `correlation` — Property
- `reset()` — Clear indicator state.
- `update(asset_return, market_return)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
