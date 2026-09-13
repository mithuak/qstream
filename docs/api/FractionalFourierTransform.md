# FractionalFourierTransform

> Fractional Fourier Transform (FrFT).

| Field | Value |
|---|---|
| Group | Experimental · Time-frequency |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import FractionalFourierTransform` |

## Signature

```python
FractionalFourierTransform(window=64, angle=1.5707963267948966, update_every=16)
```

## What it computes

Rotates the signal by angle `a` in the time-frequency plane; `a = pi/2` recovers the standard Fourier transform.

## Why use it

Locate changing frequency content in time.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `angle` | `float` | `1.5707963267948966` | Rotation angle of the fractional Fourier transform. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `SpectrumResult | None` — Frequency bins, power values, dominant frequency, and peak power..

See [SpectrumResult](/api/SpectrumResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Fractional Fourier Transform (FrFT).

```text
F_a(u) = sqrt(1 - j cot a) e^{j pi u^2 cot a}
         * int x(t) e^{j pi t^2 cot a} e^{-j 2 pi u t csc a} dt
```

Rotates the signal by angle `a` in the time-frequency plane; `a = pi/2`
recovers the standard Fourier transform.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
