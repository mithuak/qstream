# EmpiricalWaveletTransform

> Empirical Wavelet Transform (Gilles 2013).

| Field | Value |
|---|---|
| Group | Experimental · Wavelet extras |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import EmpiricalWaveletTransform` |

## Signature

```python
EmpiricalWaveletTransform(window=64, n_modes=3, update_every=16)
```

## What it computes

Builds an adaptive filter bank by detecting boundaries between spectral modes and applying the resulting empirical wavelets.

## Why use it

Analyze or denoise structure across time scales.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `n_modes` | `int` | `3` | Number of modes requested from the decomposition. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `WaveletResult | None` — Wavelet coefficients and scale information..

See [WaveletResult](/api/WaveletResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Empirical Wavelet Transform (Gilles 2013).

```text
psi_k(f) = bandpass wavelet built from detected Fourier-spectrum boundaries
```

Builds an adaptive filter bank by detecting boundaries between spectral
modes and applying the resulting empirical wavelets.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
