# AdaptiveKalman

> Adaptive 1-D Kalman filter with online measurement-noise estimation.

| Field | Value |
|---|---|
| Group | Signal · kalman / state-space / adaptive |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import AdaptiveKalman` |

## Signature

```python
AdaptiveKalman(q=1e-3, r=1e-2, adapt=0.05)
```

## What it computes

Standard Kalman update where the measurement-noise variance `R` is re-estimated online from the innovation `v = z - H x_pred`.

## Why use it

Track hidden state or predict the next observation from noisy measurements.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `q` | `float` | `1e-3` | Process-noise variance. |
| `r` | `float` | `1e-2` | Measurement-noise variance. |
| `adapt` | `float` | `0.05` | Rate at which the filter adjusts its noise estimates. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `measurement` | `float` | New noisy observation of the tracked state. |

**Output:** `StateEstimate` — Estimated state, velocity, and covariance..

## Formula and method

Adaptive 1-D Kalman filter with online measurement-noise estimation.

```text
R_hat <- R_hat + adapt * (v^2 - P_pred - R_hat)
```

Standard Kalman update where the measurement-noise variance `R` is
re-estimated online from the innovation `v = z - H x_pred`.

## Public members

- `measurement_noise` — Property
- `reset()` — Clear indicator state.
- `update(measurement)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
