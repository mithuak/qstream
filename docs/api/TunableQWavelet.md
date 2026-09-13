# TunableQWavelet

> Tunable-Q Wavelet Transform (TQWT).

| Field | Value |
|---|---|
| Group | Experimental · Wavelet extras |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import TunableQWavelet` |

## Signature

```python
TunableQWavelet(window=64, levels=4, q_factor=2.0, update_every=16)
```

## What it computes

Iterative two-channel filter bank with a tunable Q-factor; high Q for oscillatory signals, low Q for transients.

## Why use it

Analyze or denoise structure across time scales.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `levels` | `int` | `4` | Number of wavelet decomposition scales. |
| `q_factor` | `float` | `2.0` | Quality factor controlling the wavelet bandwidth. |
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

Tunable-Q Wavelet Transform (TQWT).

```text
Q = center_frequency / bandwidth
alpha = 1 - (Q - 1)/(Q + 1);  beta = 2/(Q + 1)
```

Iterative two-channel filter bank with a tunable Q-factor; high Q for
oscillatory signals, low Q for transients.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
