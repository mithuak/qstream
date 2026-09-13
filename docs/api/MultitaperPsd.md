# MultitaperPsd

> Thomson multitaper PSD (DPSS tapers).

| Field | Value |
|---|---|
| Group | Signal · spectral |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import MultitaperPsd` |

## Signature

```python
MultitaperPsd(window=256, nw=3.0, tapers=5, update_every=32)
```

## What it computes

Averages K eigenspectra computed with orthogonal Slepian (DPSS) tapers `v_k`, minimizing leakage and variance.

## Why use it

Find periodic components or track how signal power is distributed by frequency.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `256` | Number of recent samples retained for each calculation. |
| `nw` | `float` | `3.0` | Time-bandwidth product for multitaper analysis. |
| `tapers` | `int` | `5` | Number of orthogonal tapers to average. |
| `update_every` | `int` | `32` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `SpectrumResult | None` — Frequency bins, power values, dominant frequency, and peak power..

See [SpectrumResult](/api/SpectrumResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Thomson multitaper PSD (DPSS tapers).

```text
P(f) = (1/K) * sum_{k=1}^{K} |sum_t v_k[t] x_t e^{-j 2 pi f t}|^2
```

Averages K eigenspectra computed with orthogonal Slepian (DPSS) tapers
`v_k`, minimizing leakage and variance.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
