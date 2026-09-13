# RollingBetaStability

> Rolling-window beta stability (standard deviation of the rolling beta).

| Field | Value |
|---|---|
| Group | Finance · factors |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import RollingBetaStability` |

## Signature

```python
RollingBetaStability(beta_window=60, stability_window=60)
```

## What it computes

Measures how much the asset's market sensitivity varies through time.

## Why use it

Estimate an asset's sensitivity to benchmark or factor returns and attribute performance.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `beta_window` | `int` | `60` | Rolling window used to estimate each beta. |
| `stability_window` | `int` | `60` | Window over which beta stability is summarized. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `asset_return` | `float` | Return of the asset being analyzed. |
| `market_return` | `float` | Return of the market benchmark. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Rolling-window beta stability (standard deviation of the rolling beta).

```text
stability = std( beta_t ) over a stability window
```

Measures how much the asset's market sensitivity varies through time.

## Public members

- `beta` — Property
- `reset()` — Clear indicator state.
- `update(asset_return, market_return)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
