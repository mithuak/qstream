# WaveletPhaseSynchrony

> Wavelet Phase Synchrony.

| Field | Value |
|---|---|
| Group | Experimental · Wavelet extras |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import WaveletPhaseSynchrony` |

## Signature

```python
WaveletPhaseSynchrony(window=64, levels=3, update_every=16)
```

## What it computes

Phase-locking value across wavelet scales; near 1 means consistent phase difference (synchronization), near 0 means none.

## Why use it

Analyze or denoise structure across time scales.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `levels` | `int` | `3` | Number of wavelet decomposition scales. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `x` | `float` | Current observation from the first series. |
| `y` | `float` | Current observation from the second series. |

**Output:** `WaveletResult | None` — Wavelet coefficients and scale information..

See [WaveletResult](/api/WaveletResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Wavelet Phase Synchrony.

```text
PLV = |1/N sum_t e^{j (phi_x(t) - phi_y(t))}|
```

Phase-locking value across wavelet scales; near 1 means consistent phase
difference (synchronization), near 0 means none.

## Public members

- `reset()` — Clear indicator state.
- `update(x, y)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
