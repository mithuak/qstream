# AlphaBetaTracker

> Alpha-beta (g-h) tracker.

| Field | Value |
|---|---|
| Group | Signal · kalman / state-space / adaptive |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import AlphaBetaTracker` |

## Signature

```python
AlphaBetaTracker(alpha=0.5, beta=0.1, dt=1.0)
```

## What it computes

Constant-gain state estimator; a fixed-coefficient special case of the Kalman filter.

## Why use it

Track hidden state or predict the next observation from noisy measurements.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `alpha` | `float` | `0.5` | Gain applied to the measurement residual in the position estimate. |
| `beta` | `float` | `0.1` | Gain applied to the measurement residual in the velocity estimate. |
| `dt` | `float` | `1.0` | Time interval between observations. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `measurement` | `float` | New noisy observation of the tracked state. |

**Output:** `StateEstimate` — Estimated state, velocity, and covariance..

## Formula and method

Alpha-beta (g-h) tracker.

```text
x_hat <- x_hat + alpha * (z - x_hat)
v_hat <- v_hat + (beta / dt) * (z - x_hat)
x_pred = x_hat + v_hat * dt
```

Constant-gain state estimator; a fixed-coefficient special case of the
Kalman filter.

## Public members

- `reset()` — Clear indicator state.
- `update(measurement)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
