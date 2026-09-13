# ApesSpectrum

> Capon APES (Amplitude and Phase Estimation) Spectrum.

| Field | Value |
|---|---|
| Group | Experimental · Signal · Spectral and filtering |
| Source | `src/python/spectral_missing.rs` |
| Python import | `from qstream import ApesSpectrum` |

## Signature

```python
ApesSpectrum(window=64, order=4, nfft=128, update_every=16)
```

## What it computes

Narrowband filter-bank spectrum with unit-gain constraint:

## Why use it

Inspect frequency content or construct a filter from a rolling signal window.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `order` | `int` | `4` | Model, filter, or polynomial order. |
| `nfft` | `int` | `128` | FFT length or spectral grid size. |
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

Capon APES (Amplitude and Phase Estimation) Spectrum.

Narrowband filter-bank spectrum with unit-gain constraint:

```text
alpha(omega) = (a^H Q^{-1} g) / (a^H Q^{-1} a)
P(omega) = |alpha(omega)|^2
```

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
