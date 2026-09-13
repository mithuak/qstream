# WienerHopfFilter

> Wiener-Hopf Optimal Filter.

| Field | Value |
|---|---|
| Group | Experimental · Signal · Spectral and filtering |
| Source | `src/python/spectral_missing.rs` |
| Python import | `from qstream import WienerHopfFilter` |

## Signature

```python
WienerHopfFilter(window=64, order=4, update_every=8)
```

## What it computes

`update` returns the filtered value once warm; `.coefficients` exposes the designed FIR taps.

## Why use it

Inspect frequency content or construct a filter from a rolling signal window.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `order` | `int` | `4` | Model, filter, or polynomial order. |
| `update_every` | `int` | `8` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Wiener-Hopf Optimal Filter.

FIR smoother solving the Wiener-Hopf normal equations:

```text
R h = p
R[i][j] = r_xx(|i - j|),  p[i] = r_xd(i)
y_t = sum_i h[i] x_{t-i}
```

`update` returns the filtered value once warm; `.coefficients` exposes the
designed FIR taps.

## Public members

- `coefficients` — Property
- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
