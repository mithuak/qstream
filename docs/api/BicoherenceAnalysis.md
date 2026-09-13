# BicoherenceAnalysis

> Bicoherence analysis.

| Field | Value |
|---|---|
| Group | Experimental · Higher-order spectra |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import BicoherenceAnalysis` |

## Signature

```python
BicoherenceAnalysis(window=64, nfft=64, update_every=16)
```

## What it computes

Normalized bispectrum in [0, 1]; 1 indicates perfect quadratic coupling.

## Why use it

Inspect nonlinear or non-Gaussian interactions among frequencies.

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

**Output:** `PyBicoherenceResult | None`.

See [PyBicoherenceResult](/api/PyBicoherenceResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Bicoherence analysis.

```text
b^2(f1, f2) = |B(f1, f2)|^2 / (E|X(f1)X(f2)|^2 * E|X(f1+f2)|^2)
```

Normalized bispectrum in [0, 1]; 1 indicates perfect quadratic coupling.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
