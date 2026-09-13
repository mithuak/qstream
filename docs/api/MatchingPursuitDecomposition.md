# MatchingPursuitDecomposition

> Matching Pursuit greedy sparse decomposition.

| Field | Value |
|---|---|
| Group | Experimental · Decomposition |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import MatchingPursuitDecomposition` |

## Signature

```python
MatchingPursuitDecomposition(window=64, n_atoms=5, update_every=16)
```

## What it computes

Iteratively selects the Gabor atom `g_k` best correlated with the residual `R_k`.

## Why use it

Separate a composite signal into simpler oscillatory or sparse components.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `n_atoms` | `int` | `5` | Maximum number of atoms selected by matching pursuit. |
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

Matching Pursuit greedy sparse decomposition.

```text
x = sum_k <R_k, g_k> g_k + R_{K+1}
```

Iteratively selects the Gabor atom `g_k` best correlated with the residual
`R_k`.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
