# SpectralShapeResult

> Spectral-shape features.

| Field | Value |
|---|---|
| Group | Result types |
| Source | `src/python/result_types.rs` |
| Python import | `from qstream import SpectralShapeResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `centroid` | `float` | Power-weighted center frequency. |
| `bandwidth` | `float` | Spread of spectral power around the centroid. |
| `entropy` | `float` | Entropy or concentration measure of the output distribution. |
| `rolloff` | `float` | Frequency below which a specified fraction of total power lies. |
| `flatness` | `float` | Ratio of geometric to arithmetic mean spectral power. |
| `dominant_frequency` | `float` | Frequency of the strongest estimated component. |

## Formula and method

Spectral-shape features.

## Public members

- `bandwidth` — Property
- `centroid` — Property
- `dominant_frequency` — Property
- `entropy` — Property
- `flatness` — Property
- `rolloff` — Property

[Back to the API catalog](/api/).
