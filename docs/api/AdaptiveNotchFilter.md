# AdaptiveNotchFilter

> Adaptive notch filter frequency estimator. Locks onto and suppresses a dominant sinusoid, returning the estimated normalized frequency (cycles per sample) and the notched output each tick.

| Field | Value |
|---|---|
| Group | Signal · filters |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import AdaptiveNotchFilter` |

## Signature

```python
AdaptiveNotchFilter(rho=0.95, mu=1e-3, freq0=0.05)
```

## What it computes

A second-order notch with a gradient-adapted center frequency `w0_hat`.

## Why use it

Smooth or adapt a noisy series before measurement or prediction.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `rho` | `float` | `0.95` | Pole radius controlling the notch bandwidth. |
| `mu` | `float` | `1e-3` | Adaptation step size for the filter. |
| `freq0` | `float` | `0.05` | Initial or target normalized frequency for the adaptive tracker. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `FrequencyEstimate` — Estimated frequency and filtered sample..

## Formula and method

Adaptive notch filter frequency estimator. Locks onto and suppresses a
dominant sinusoid, returning the estimated normalized frequency (cycles per
sample) and the notched output each tick.

```text
H(z) = (1 - 2 rho cos(w0) z^{-1} + rho^2 z^{-2})
       / (1 - 2 rho cos(w0_hat) z^{-1} + rho^2 z^{-2})
w0_hat <- w0_hat - mu * dE/dw0_hat      (gradient descent)
```

A second-order notch with a gradient-adapted center frequency `w0_hat`.

## Public members

- `frequency` — Property
- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
