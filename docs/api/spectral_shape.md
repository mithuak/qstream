# spectral_shape

> Compute spectral-shape features from a [`SpectrumResult`].

| Field | Value |
|---|---|
| Group | Signal · spectral |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import spectral_shape` |

## Signature

```python
spectral_shape(spectrum)
```

## What it computes

Compute spectral-shape features from a [`SpectrumResult`].

## Why use it

Find periodic components or track how signal power is distributed by frequency.

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `spectrum` | `&SpectrumResult` | SpectrumResult to summarize into shape features. |

**Output:** `SpectralShapeResult`.

See [SpectralShapeResult](/api/SpectralShapeResult) for its output fields.

## Formula and method

Compute spectral-shape features from a [`SpectrumResult`].

```text
centroid = sum f P(f) / sum P(f)
entropy  = -sum p log p / log N
flatness = exp(mean log P) / mean P
rolloff  = f where cumulative energy reaches 85%
```

[Back to the API catalog](/api/).
