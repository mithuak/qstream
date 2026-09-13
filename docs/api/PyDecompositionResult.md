# PyDecompositionResult

> Structured output returned by a qstream calculation.

| Field | Value |
|---|---|
| Group | Experimental · Result types |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import PyDecompositionResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `coefficients` | `list[float]` | Model or transform coefficients. |
| `n_modes` | `int` | Number of extracted signal modes. |
| `mode_frequencies` | `list[float]` | Estimated frequency of each extracted mode. |

## Public members

- `coefficients` — Property
- `mode_frequencies` — Property
- `n_modes` — Property

[Back to the API catalog](/api/).
