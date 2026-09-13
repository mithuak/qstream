# PyCumulantResult

> Structured output returned by a qstream calculation.

| Field | Value |
|---|---|
| Group | Experimental · Result types |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import PyCumulantResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `skewness` | `float` | Third-order standardized asymmetry measure. |
| `kurtosis` | `float` | Fourth-order tail or peakedness measure. |
| `nonlinearity_index` | `float` | Summary measure of nonlinearity from higher-order cumulants. |

## Public members

- `kurtosis` — Property
- `nonlinearity_index` — Property
- `skewness` — Property

[Back to the API catalog](/api/).
