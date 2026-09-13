# ConstantQTransform

> Constant-Q Transform (CQT).

| Field | Value |
|---|---|
| Group | Experimental · Time-frequency |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import ConstantQTransform` |

## Signature

```python
ConstantQTransform(window=64, n_bins=24, f_min=0.01, f_max=0.4, update_every=16)
```

## What it computes

Logarithmically spaced frequency bins with constant Q (center frequency / bandwidth ratio).

## Why use it

Locate changing frequency content in time.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `n_bins` | `int` | `24` | Number of frequency bins to analyze. |
| `f_min` | `float` | `0.01` | Lowest analyzed frequency. |
| `f_max` | `float` | `0.4` | Highest analyzed frequency. |
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

Constant-Q Transform (CQT).

```text
X(f_k) = sum_n x[n] w(n, f_k) e^{-j 2 pi f_k n}
f_k = f_min * 2^{k/bins}
```

Logarithmically spaced frequency bins with constant Q (center frequency /
bandwidth ratio).

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
