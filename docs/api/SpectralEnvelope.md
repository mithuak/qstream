# SpectralEnvelope

> Spectral Envelope.

| Field | Value |
|---|---|
| Group | Experimental · Signal · Spectral and filtering |
| Source | `src/python/spectral_missing.rs` |
| Python import | `from qstream import SpectralEnvelope` |

## Signature

```python
SpectralEnvelope(window=64, cepstral_order=8, update_every=16)
```

## What it computes

Envelope via cepstral smoothing of the log-magnitude spectrum:

## Why use it

Inspect frequency content or construct a filter from a rolling signal window.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `cepstral_order` | `int` | `8` | Number of cepstral terms retained in the envelope. |
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

Spectral Envelope.

Envelope via cepstral smoothing of the log-magnitude spectrum:

```text
env(f) = exp( IFFT( lowpass( FFT( log|X(f)| ) ) ) )
```

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
