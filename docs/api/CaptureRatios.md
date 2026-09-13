# CaptureRatios

> Up/down capture ratios versus a benchmark.

| Field | Value |
|---|---|
| Group | Finance · performance |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import CaptureRatios` |

## Signature

```python
CaptureRatios(period)
```

## What it computes

Asset participation in benchmark up and down moves, respectively.

## Why use it

Evaluate return quality relative to risk, downside, or a benchmark.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `required` | Rolling length or effective horizon; see the formula for this indicator's convention. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `asset_return` | `float` | Return of the asset being analyzed. |
| `benchmark_return` | `float` | Return of the comparison benchmark. |

**Output:** `CaptureResult | None` — Up-market and down-market capture ratios..

See [CaptureResult](/api/CaptureResult) for its output fields.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Up/down capture ratios versus a benchmark.

```text
up_capture = mean(r_a | r_b > 0) / mean(r_b | r_b > 0)
down_capture = mean(r_a | r_b < 0) / mean(r_b | r_b < 0)
```

Asset participation in benchmark up and down moves, respectively.

## Public members

- `reset()` — Clear indicator state.
- `update(asset_return, benchmark_return)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
