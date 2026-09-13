# CaponSpectrum

> Capon (MVDR) spectral estimator.

| Field | Value |
|---|---|
| Group | Experimental · Spectral heavy methods |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import CaponSpectrum` |

## Signature

```python
CaponSpectrum(window, ar_order, nfft, update_every)
```

## What it computes

Minimum-variance distortionless-response spectrum from the inverse autocorrelation matrix.

## Why use it

Resolve closely spaced spectral components when a simpler periodogram is insufficient.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `required` | Number of recent samples retained for each calculation. |
| `ar_order` | `int` | `required` | Order of the autoregressive model used in the spectrum estimate. |
| `nfft` | `int` | `required` | FFT length or spectral grid size. |
| `update_every` | `int` | `required` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `SpectrumResult | None` — Frequency bins, power values, dominant frequency, and peak power..

See [SpectrumResult](/api/SpectrumResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Capon (MVDR) spectral estimator.

```text
P(f) = 1 / ( e(f)^H R^{-1} e(f) )
```

Minimum-variance distortionless-response spectrum from the inverse
autocorrelation matrix.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
