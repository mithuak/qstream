# PyBispectrumResult

> Structured output returned by a qstream calculation.

| Field | Value |
|---|---|
| Group | Experimental · Result types |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import PyBispectrumResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `values` | `list[float]` | Computed spectral or higher-order statistic values. |
| `frequencies` | `list[float]` | Frequency coordinates for the spectral values. |
| `entropy` | `float` | Entropy or concentration measure of the output distribution. |
| `total_coupling` | `float` | Aggregate strength of estimated frequency coupling. |

## Public members

- `entropy` — Property
- `frequencies` — Property
- `total_coupling` — Property
- `values` — Property

[Back to the API catalog](/api/).
