# FastICA

> FastICA Blind Source Separation.

| Field | Value |
|---|---|
| Group | Experimental · FastICA |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import FastICA` |

## Signature

```python
FastICA(n_signals=2, n_components=2, window=64, update_every=16)
```

## What it computes

Recovers independent components by maximizing non-Gaussianity (negentropy approximation).

## Why use it

Separate mixed signals into approximately independent components.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `n_signals` | `int` | `2` | Number of simultaneous input signals. |
| `n_components` | `int` | `2` | Number of independent components to return. |
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `signals` | `list[float]` | Vector of simultaneous signal observations. |

**Output:** `list[list[float]] | None`.

A `None` element means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

FastICA Blind Source Separation.

```text
w <- E{x g(w^T x)} - E{g'(w^T x)} w   (fixed-point iteration)
s = W x
```

Recovers independent components by maximizing non-Gaussianity (negentropy
approximation).

## Public members

- `reset()` — Clear indicator state.
- `update(signals)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
