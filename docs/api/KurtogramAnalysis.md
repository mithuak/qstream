# KurtogramAnalysis

> Kurtogram (spectral kurtosis).

| Field | Value |
|---|---|
| Group | Experimental · Time-frequency |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import KurtogramAnalysis` |

## Signature

```python
KurtogramAnalysis(window=64, nfft=64, update_every=16)
```

## What it computes

Frequency-resolved kurtosis; high values flag bands containing transients or impulsive components.

## Why use it

Locate changing frequency content in time.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
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

Kurtogram (spectral kurtosis).

```text
K(f) = E|X(f)|^4 / (E|X(f)|^2)^2 - 3
```

Frequency-resolved kurtosis; high values flag bands containing transients
or impulsive components.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
