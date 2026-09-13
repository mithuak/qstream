# LatticePredictionErrorFilter

> Lattice prediction-error filter (outputs the whitened residual).

| Field | Value |
|---|---|
| Group | Signal · prediction |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import LatticePredictionErrorFilter` |

## Signature

```python
LatticePredictionErrorFilter(window=64, order=8, update_every=8)
```

## What it computes

Forward/backward lattice recursion driven by Burg reflection coefficients `k_m`; outputs the order-p forward prediction error.

## Why use it

Model short-horizon dynamics or forecast a future sample.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `order` | `int` | `8` | Model, filter, or polynomial order. |
| `update_every` | `int` | `8` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Lattice prediction-error filter (outputs the whitened residual).

```text
f_m = f_{m-1} - k_m b_{m-1}
b_m = b_{m-1} - k_m f_{m-1}
residual = f_p = x_t - sum_i a_i x_{t-i}
```

Forward/backward lattice recursion driven by Burg reflection coefficients
`k_m`; outputs the order-p forward prediction error.

## Public members

- `reflection_coefficients` — Property
- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
