# TrackingError

> Tracking error: rolling standard deviation of active returns.

| Field | Value |
|---|---|
| Group | Finance · factors |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import TrackingError` |

## Signature

```python
TrackingError(period)
```

## What it computes

Volatility of the return difference between an asset and its benchmark.

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

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Tracking error: rolling standard deviation of active returns.

```text
TE = std( r_a - r_b )
```

Volatility of the return difference between an asset and its benchmark.

## Public members

- `reset()` — Clear indicator state.
- `update(asset_return, benchmark_return)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
