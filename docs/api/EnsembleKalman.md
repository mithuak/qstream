# EnsembleKalman

> Ensemble Kalman Filter (EnKF).

| Field | Value |
|---|---|
| Group | Experimental · Heavy Kalman |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import EnsembleKalman` |

## Signature

```python
EnsembleKalman(n_members=50, state_dim=2, process_noise=0.01, measurement_noise=0.1)
```

## What it computes

Monte Carlo Kalman filter replacing the covariance with the sample covariance of an ensemble.

## Why use it

Estimate latent state when a simpler linear Gaussian filter is not suitable.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `n_members` | `int` | `50` | Number of members in the state ensemble. |
| `state_dim` | `int` | `2` | Number of latent state dimensions. |
| `process_noise` | `float` | `0.01` | Process-noise variance or scale. |
| `measurement_noise` | `float` | `0.1` | Measurement-noise variance or scale. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `measurement` | `float` | New noisy observation of the tracked state. |

**Output:** `StateEstimate` — Estimated state, velocity, and covariance..

## Formula and method

Ensemble Kalman Filter (EnKF).

```text
x_hat = (1/N) sum_i x^i
P = (1/(N-1)) sum_i (x^i - x_hat)(x^i - x_hat)^T
x^i <- x^i + K (y - H x^i)
```

Monte Carlo Kalman filter replacing the covariance with the sample
covariance of an ensemble.

## Public members

- `reset()` — Clear indicator state.
- `update(measurement)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
