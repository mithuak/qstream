# SquareRootKalman

> Scalar square-root (covariance-factor) Kalman filter.

| Field | Value |
|---|---|
| Group | Signal · kalman / state-space / adaptive |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import SquareRootKalman` |

## Signature

```python
SquareRootKalman(q=1e-3, r=1e-2, s0=1.0)
```

## What it computes

Propagates the square-root of the covariance instead of the covariance itself for improved numerical stability.

## Why use it

Track hidden state or predict the next observation from noisy measurements.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `q` | `float` | `1e-3` | Process-noise variance. |
| `r` | `float` | `1e-2` | Measurement-noise variance. |
| `s0` | `float` | `1.0` | Initial covariance factor. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `measurement` | `float` | New noisy observation of the tracked state. |

**Output:** `StateEstimate` — Estimated state, velocity, and covariance..

## Formula and method

Scalar square-root (covariance-factor) Kalman filter.

```text
P = S * S^T     (covariance factor)
S_est = sqrt((1 - K H) S_pred^2 + K^2 R)
```

Propagates the square-root of the covariance instead of the covariance
itself for improved numerical stability.

## Public members

- `reset()` — Clear indicator state.
- `update(measurement)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
