# MultivariateSpectralAnalysis

> Multivariate Spectral Analysis.

| Field | Value |
|---|---|
| Group | Experimental · Signal · Spectral and filtering |
| Source | `src/python/spectral_missing.rs` |
| Python import | `from qstream import MultivariateSpectralAnalysis` |

## Signature

```python
MultivariateSpectralAnalysis(window=64, embedding=3, nfft=64, update_every=16)
```

## What it computes

Total power spectrum from time-delay embedding, as the trace of the cross-spectral matrix:

## Why use it

Inspect frequency content or construct a filter from a rolling signal window.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `embedding` | `int` | `3` | Number of delayed coordinates in the multivariate embedding. |
| `nfft` | `int` | `64` | FFT length or spectral grid size. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `SpectrumResult | None` — Frequency bins, power values, dominant frequency, and peak power..

See [SpectrumResult](/api/SpectrumResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Multivariate Spectral Analysis.

Total power spectrum from time-delay embedding, as the trace of the
cross-spectral matrix:

```text
S(f) = sum_{i,j} S_ij(f),  S_ij(f) = E[X_i(f) X_j*(f)]
```

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
