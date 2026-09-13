# LmdDecomposition

> Local Mean Decomposition (LMD).

| Field | Value |
|---|---|
| Group | Experimental · Decomposition |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import LmdDecomposition` |

## Signature

```python
LmdDecomposition(window=64, max_pfs=4, update_every=16)
```

## What it computes

Decomposes into product functions of a smoothed envelope `a_k` and a purely frequency-modulated signal `s_k`.

## Why use it

Separate a composite signal into simpler oscillatory or sparse components.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `max_pfs` | `int` | `4` | Maximum product functions to extract. |
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

Local Mean Decomposition (LMD).

```text
x_t = sum_k PF_k(t) ;  PF_k = a_k(t) * s_k(t)
```

Decomposes into product functions of a smoothed envelope `a_k` and a purely
frequency-modulated signal `s_k`.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
