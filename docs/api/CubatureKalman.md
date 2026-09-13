# CubatureKalman

> Cubature Kalman Filter (CKF).

| Field | Value |
|---|---|
| Group | Experimental · Heavy Kalman |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import CubatureKalman` |

## Signature

```python
CubatureKalman(state_dim=2, process_noise=0.01, measurement_noise=0.1)
```

## What it computes

Deterministic sampling (cubature rule) for nonlinear filtering; accurate to third order.

## Why use it

Estimate latent state when a simpler linear Gaussian filter is not suitable.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `state_dim` | `int` | `2` | Number of latent state dimensions. |
| `process_noise` | `float` | `0.01` | Process-noise variance or scale. |
| `measurement_noise` | `float` | `0.1` | Measurement-noise variance or scale. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `measurement` | `float` | New noisy observation of the tracked state. |

**Output:** `StateEstimate` — Estimated state, velocity, and covariance..

## Formula and method

Cubature Kalman Filter (CKF).

```text
X_i = mean +/- sqrt(n P) e_i     (cubature points)
mean = (1/2n) sum_i f(X_i)
P = (1/2n) sum_i (f(X_i) - mean)(f(X_i) - mean)^T + Q
```

Deterministic sampling (cubature rule) for nonlinear filtering; accurate
to third order.

## Public members

- `reset()` — Clear indicator state.
- `update(measurement)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
