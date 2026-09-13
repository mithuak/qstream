# ShortTimeFourierTransform

> Short-Time Fourier Transform producing a running spectrogram.

| Field | Value |
|---|---|
| Group | Signal · time-frequency (STFT / Hilbert) |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import ShortTimeFourierTransform` |

## Signature

```python
ShortTimeFourierTransform(window=256, hop=128, fs=1.0, window_type="hann", max_frames=64)
```

## What it computes

Each hop returns the magnitude spectrum of the current windowed frame; `.spectrogram()` returns the accumulated time-frequency matrix `(frames, bins, flat_values)`.

## Why use it

Track how oscillation, envelope, or frequency changes over time.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `256` | Number of recent samples retained for each calculation. |
| `hop` | `int` | `128` | Number of samples between adjacent transform frames. |
| `fs` | `float` | `1.0` | Sampling frequency in samples per unit time. |
| `window_type` | `str` | `"hann"` | Taper applied to the signal window. |
| `max_frames` | `int` | `64` | Maximum spectrogram frames retained. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `SpectrumResult | None` — Frequency bins, power values, dominant frequency, and peak power..

See [SpectrumResult](/api/SpectrumResult) for its output fields.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Short-Time Fourier Transform producing a running spectrogram.

```text
X(t, f) = sum_k w[k] x_{t+k} e^{-j 2 pi f k}
```

Each hop returns the magnitude spectrum of the current windowed frame;
`.spectrogram()` returns the accumulated time-frequency matrix
`(frames, bins, flat_values)`.

## Public members

- `reset()` — Clear indicator state.
- `spectrogram()` — Accumulated spectrogram as `(n_frames, n_bins, flat_magnitudes)`.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
