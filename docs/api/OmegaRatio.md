# OmegaRatio

> Omega ratio above a threshold.

| Field | Value |
|---|---|
| Group | Finance · performance |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import OmegaRatio` |

## Signature

```python
OmegaRatio(period=252, threshold=0.0)
```

## What it computes

Gain-to-loss ratio measured relative to a threshold `tau`.

## Why use it

Evaluate return quality relative to risk, downside, or a benchmark.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `252` | Rolling length or effective horizon; see the formula for this indicator's convention. |
| `threshold` | `float` | `0.0` | Decision threshold for the detector or filter. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Omega ratio above a threshold.

```text
Omega(tau) = sum( max(0, r - tau) ) / sum( max(0, tau - r) )
```

Gain-to-loss ratio measured relative to a threshold `tau`.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
