# KalmanFilter

> Linear Kalman filter (constant-velocity model) tracking a scalar series.

| Field | Value |
|---|---|
| Group | Signal · kalman / state-space / adaptive |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import KalmanFilter` |

## Signature

```python
KalmanFilter(dt=1.0, q=1e-3, r=1.0, p0=1.0)
```

## What it computes

Optimal linear state estimator for a constant-velocity (position + velocity) motion model.

## Why use it

Track hidden state or predict the next observation from noisy measurements.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `dt` | `float` | `1.0` | Time interval between observations. |
| `q` | `float` | `1e-3` | Process-noise variance. |
| `r` | `float` | `1.0` | Measurement-noise variance. |
| `p0` | `float` | `1.0` | Initial state covariance. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `measurement` | `float` | New noisy observation of the tracked state. |

**Output:** `StateEstimate` — Estimated state, velocity, and covariance..

## Formula and method

Linear Kalman filter (constant-velocity model) tracking a scalar series.

```text
x_pred = F x_est
P_pred = F P_est F^T + Q
K = P_pred H^T (H P_pred H^T + R)^{-1}
x_est = x_pred + K (z - H x_pred)
P_est = (I - K H) P_pred
```

Optimal linear state estimator for a constant-velocity (position +
velocity) motion model.

## Public members

- `reset()` — Clear indicator state.
- `update(measurement)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
