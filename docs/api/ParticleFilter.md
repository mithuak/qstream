# ParticleFilter

> Bootstrap Particle Filter (Sequential Importance Resampling).

| Field | Value |
|---|---|
| Group | Experimental · Heavy Kalman |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import ParticleFilter` |

## Signature

```python
ParticleFilter(n_particles=200, process_noise=0.01, measurement_noise=0.1, state_min=..., state_max=5.0)
```

## What it computes

Monte Carlo state estimator for nonlinear/non-Gaussian systems, with systematic resampling when the effective sample size is low.

## Why use it

Estimate latent state when a simpler linear Gaussian filter is not suitable.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `n_particles` | `int` | `200` | Number of particles used to approximate the state distribution. |
| `process_noise` | `float` | `0.01` | Process-noise variance or scale. |
| `measurement_noise` | `float` | `0.1` | Measurement-noise variance or scale. |
| `state_min` | `float` | `...` | Lower bound of the particle state range. |
| `state_max` | `float` | `5.0` | Upper bound of the particle state range. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `measurement` | `float` | New noisy observation of the tracked state. |

**Output:** `StateEstimate` — Estimated state, velocity, and covariance..

## Formula and method

Bootstrap Particle Filter (Sequential Importance Resampling).

```text
x_t^i ~ p(x_t | x_{t-1}^i)
w_t^i ~ p(y_t | x_t^i)
x_hat = sum_i w_t^i x_t^i
```

Monte Carlo state estimator for nonlinear/non-Gaussian systems, with
systematic resampling when the effective sample size is low.

## Public members

- `reset()` — Clear indicator state.
- `update(measurement)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
