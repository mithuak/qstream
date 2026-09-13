# PyBicoherenceResult

> Structured output returned by a qstream calculation.

| Field | Value |
|---|---|
| Group | Experimental · Result types |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import PyBicoherenceResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `values` | `list[float]` | Computed spectral or higher-order statistic values. |
| `frequencies` | `list[float]` | Frequency coordinates for the spectral values. |
| `peak` | `float` | Largest estimated bicoherence value. |

## Public members

- `frequencies` — Property
- `peak` — Property
- `values` — Property

[Back to the API catalog](/api/).
