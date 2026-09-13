# LpcPredictor

> Streaming LPC / Levinson-Durbin predictor.

| Field | Value |
|---|---|
| Group | Signal · prediction |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import LpcPredictor` |

## Signature

```python
LpcPredictor(window=64, order=8, update_every=8)
```

## What it computes

Linear predictive coding: AR(p) coefficients via Levinson-Durbin, with the one-step prediction output every tick.

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

**Output:** `PredictionResult | None` — Prediction and fitted coefficients..

See [PredictionResult](/api/PredictionResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Streaming LPC / Levinson-Durbin predictor.

```text
x_t = sum_{i=1}^{p} a_i x_{t-i} + e_t
prediction = sum_i a_i x_{t-i}
```

Linear predictive coding: AR(p) coefficients via Levinson-Durbin, with the
one-step prediction output every tick.

## Public members

- `coefficients` — Property
- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
