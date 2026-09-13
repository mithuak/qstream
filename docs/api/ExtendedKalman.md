# ExtendedKalman

> Extended Kalman filter (constant-velocity model, analytic Jacobian).

| Field | Value |
|---|---|
| Group | Signal · kalman / state-space / adaptive |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import ExtendedKalman` |

## Signature

```python
ExtendedKalman(dt=1.0, q=1e-3, r=1.0, p0=1.0, nonlinear=False)
```

## What it computes

Kalman filter linearized around the current estimate; handles mildly nonlinear state/measurement models.

## Why use it

Track hidden state or predict the next observation from noisy measurements.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `dt` | `float` | `1.0` | Time interval between observations. |
| `q` | `float` | `1e-3` | Process-noise variance. |
| `r` | `float` | `1.0` | Measurement-noise variance. |
| `p0` | `float` | `1.0` | Initial state covariance. |
| `nonlinear` | `bool` | `False` | Enable the built-in nonlinear state/observation model. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `measurement` | `float` | New noisy observation of the tracked state. |

**Output:** `StateEstimate` — Estimated state, velocity, and covariance..

## Formula and method

Extended Kalman filter (constant-velocity model, analytic Jacobian).

```text
x_pred = f(x_est)
F = df/dx |_{x_est}     (Jacobian)
P_pred = F P_est F^T + Q
K = P_pred H^T (H P_pred H^T + R)^{-1}
x_est = x_pred + K (z - h(x_pred))
```

Kalman filter linearized around the current estimate; handles mildly
nonlinear state/measurement models.

## Public members

- `reset()` — Clear indicator state.
- `update(measurement)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
