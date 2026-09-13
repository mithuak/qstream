# MatrixPencilSpectrum

> Matrix Pencil (Hua-Sarkar) spectrum.

| Field | Value |
|---|---|
| Group | Experimental · Spectral heavy methods |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import MatrixPencilSpectrum` |

## Signature

```python
MatrixPencilSpectrum(window, n_sources, nfft, update_every)
```

## What it computes

Generalized eigenvalue decomposition of two Hankel matrices.

## Why use it

Resolve closely spaced spectral components when a simpler periodogram is insufficient.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `required` | Number of recent samples retained for each calculation. |
| `n_sources` | `int` | `required` | Number of spectral sources to estimate. |
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

Matrix Pencil (Hua-Sarkar) spectrum.

```text
Y2 - z Y1 = 0 ;  z_i = eig( Y1^+ Y2 )
```

Generalized eigenvalue decomposition of two Hankel matrices.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Consume a sequence and return one result per input.

[Back to the API catalog](/api/).
