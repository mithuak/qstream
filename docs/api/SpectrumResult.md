# SpectrumResult

> One-sided spectral estimate.

| Field | Value |
|---|---|
| Group | Result types |
| Source | `src/python/result_types.rs` |
| Python import | `from qstream import SpectrumResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `frequencies` | `list[float]` | Frequency coordinates for the spectral values. |
| `power` | `list[float]` | Estimated power at each frequency. |
| `dominant_frequency` | `float \| None` | Frequency of the strongest estimated component. |
| `peak_power` | `float \| None` | Power of the strongest estimated component. |

## Formula and method

One-sided spectral estimate.

## Public members

- `dominant_frequency` — Property
- `frequencies` — Property
- `peak_power` — Property
- `power` — Property

[Back to the API catalog](/api/).
