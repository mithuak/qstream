# Goertzel

> Goertzel DFT at a set of target normalized frequencies.

| Field | Value |
|---|---|
| Group | Signal · spectral |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import Goertzel` |

## Signature

```python
Goertzel(frequencies, block=64)
```

## What it computes

Efficient single-bin DFT recursion; ideal for detecting a few known tones.

## Why use it

Find periodic components or track how signal power is distributed by frequency.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `frequencies` | `list[float]` | `required` | Frequencies at which the detector evaluates power. |
| `block` | `int` | `64` | Number of samples in one Goertzel analysis block. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `SpectrumResult | None` — Frequency bins, power values, dominant frequency, and peak power..

See [SpectrumResult](/api/SpectrumResult) for its output fields.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Goertzel DFT at a set of target normalized frequencies.

```text
s_k = x_t + 2 cos(2 pi f_k) s_{k-1} - s_{k-2}
|X(f_k)|^2 = s_k^2 + s_{k-1}^2 - 2 cos(2 pi f_k) s_k s_{k-1}
```

Efficient single-bin DFT recursion; ideal for detecting a few known tones.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
