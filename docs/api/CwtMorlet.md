# CwtMorlet

> Morlet continuous wavelet transform evaluated at the latest sample.

| Field | Value |
|---|---|
| Group | Signal · wavelets |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import CwtMorlet` |

## Signature

```python
CwtMorlet(window=64, scales=..., update_every=16)
```

## What it computes

Inner product of the signal with a scaled Morlet mother wavelet at the given dyadic scales `a`.

## Why use it

Inspect signal behavior at more than one time scale.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `scales` | `list[float]` | `...` | Wavelet scales to evaluate. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `WaveletResult | None` — Wavelet coefficients and scale information..

See [WaveletResult](/api/WaveletResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Morlet continuous wavelet transform evaluated at the latest sample.

```text
W(a) = sum_t x_t * (1/sqrt(a)) psi*((t - tau)/a)
psi(t) = pi^{-1/4} e^{j w0 t} e^{-t^2/2}
```

Inner product of the signal with a scaled Morlet mother wavelet at the
given dyadic scales `a`.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
