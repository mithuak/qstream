# UnscentedKalman

> Unscented Kalman filter (constant-velocity model).

| Field | Value |
|---|---|
| Group | Signal · kalman / state-space / adaptive |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import UnscentedKalman` |

## Signature

```python
UnscentedKalman(dt=1.0, q=1e-3, r=1.0, p0=1.0, nonlinear=False)
```

## What it computes

Deterministic sampling (unscented transform) of a nonlinear state model; accurate to second order for the propagated mean/covariance.

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

Unscented Kalman filter (constant-velocity model).

```text
X_i = mean +/- sqrt((n+kappa) P)   (sigma points)
Y_i = f(X_i)
mean = sum w_i Y_i ; P = sum w_i (Y_i - mean)(Y_i - mean)^T + Q
```

Deterministic sampling (unscented transform) of a nonlinear state model;
accurate to second order for the propagated mean/covariance.

## Public members

- `reset()` — Clear indicator state.
- `update(measurement)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
