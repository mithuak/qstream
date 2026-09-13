# EmdDecomposition

> Empirical Mode Decomposition (EMD / Hilbert-Huang).

| Field | Value |
|---|---|
| Group | Experimental · Decomposition |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import EmdDecomposition` |

## Signature

```python
EmdDecomposition(window=64, max_imfs=4, update_every=16)
```

## What it computes

Sifting loop extracts intrinsic mode functions (zero-mean, one extrema count) adaptively.

## Why use it

Separate a composite signal into simpler oscillatory or sparse components.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `max_imfs` | `int` | `4` | Maximum intrinsic mode functions to extract. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `PyDecompositionResult | None`.

See [PyDecompositionResult](/api/PyDecompositionResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Empirical Mode Decomposition (EMD / Hilbert-Huang).

```text
x_t = sum_k IMF_k(t) + r(t)
```

Sifting loop extracts intrinsic mode functions (zero-mean, one extrema
count) adaptively.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
