# MultipleCoherence

> Multiple (Multitaper) Coherence.

| Field | Value |
|---|---|
| Group | Experimental · Signal · Spectral and filtering |
| Source | `src/python/spectral_missing.rs` |
| Python import | `from qstream import MultipleCoherence` |

## Signature

```python
MultipleCoherence(window=64, nw=3.0, tapers=4, update_every=32)
```

## What it computes

Values are in `[0, 1]`.

## Why use it

Inspect frequency content or construct a filter from a rolling signal window.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `nw` | `float` | `3.0` | Time-bandwidth product for multitaper analysis. |
| `tapers` | `int` | `4` | Number of orthogonal tapers to average. |
| `update_every` | `int` | `32` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `x` | `float` | Current observation from the first series. |
| `y` | `float` | Current observation from the second series. |

**Output:** `SpectrumResult | None` — Frequency bins, power values, dominant frequency, and peak power..

See [SpectrumResult](/api/SpectrumResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Multiple (Multitaper) Coherence.

Magnitude-squared coherence averaged over DPSS eigenspectra:

```text
gamma^2(f) = |sum_k S_xy^(k)(f)|^2
             / (sum_k S_xx^(k)(f) * sum_k S_yy^(k)(f))
```

Values are in `[0, 1]`.

## Public members

- `reset()` — Clear indicator state.
- `update(x, y)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
