# WienerFilter

> Local (Wiener2-style) adaptive denoiser.

| Field | Value |
|---|---|
| Group | Signal · filters |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import WienerFilter` |

## Signature

```python
WienerFilter(window=9, noise_window=32)
```

## What it computes

Per-pixel (per-sample) Wiener filtering: estimates local mean `mu` and variance `sigma^2`, and shrinks toward the mean by the estimated noise variance `nu^2`.

## Why use it

Smooth or adapt a noisy series before measurement or prediction.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `9` | Number of recent samples retained for each calculation. |
| `noise_window` | `int` | `32` | Window used to estimate local noise level. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Local (Wiener2-style) adaptive denoiser.

```text
y = mu + max(0, sigma^2 - nu^2) / sigma^2 * (x - mu)
```

Per-pixel (per-sample) Wiener filtering: estimates local mean `mu` and
variance `sigma^2`, and shrinks toward the mean by the estimated
noise variance `nu^2`.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
