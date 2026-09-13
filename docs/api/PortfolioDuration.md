# PortfolioDuration

> Aggregate portfolio duration.

| Field | Value |
|---|---|
| Group | Finance · portfolio |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import PortfolioDuration` |

## Signature

```python
PortfolioDuration(n_assets)
```

## What it computes

Weighted average of asset durations.

## Why use it

Combine asset observations into portfolio-level returns or risk measures.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `n_assets` | `int` | `required` | Number of assets expected in each input vector. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `durations` | `list[float]` | Duration of each asset in the portfolio. |
| `weights` | `list[float]` | Portfolio weights aligned with the asset vector. |

**Output:** `float`.

## Formula and method

Aggregate portfolio duration.

```text
D_p = sum_i w_i D_i
```

Weighted average of asset durations.

## Public members

- `reset()` — Clear indicator state.
- `update(durations, weights)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
