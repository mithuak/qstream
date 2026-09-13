# VmdDecomposition

> Variational Mode Decomposition (VMD).

| Field | Value |
|---|---|
| Group | Experimental · Decomposition |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import VmdDecomposition` |

## Signature

```python
VmdDecomposition(window=64, n_modes=3, alpha=2000.0, update_every=16)
```

## What it computes

Non-recursive decomposition into K band-limited intrinsic modes via ADMM.

## Why use it

Separate a composite signal into simpler oscillatory or sparse components.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `n_modes` | `int` | `3` | Number of modes requested from the decomposition. |
| `alpha` | `float` | `2000.0` | Bandwidth penalty in the VMD optimization. |
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

Variational Mode Decomposition (VMD).

```text
min_{u_k, w_k} sum_k || d_t[(delta + j/pi t) * u_k] e^{-j w_k t} ||^2
s.t. sum_k u_k = f
```

Non-recursive decomposition into K band-limited intrinsic modes via ADMM.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
