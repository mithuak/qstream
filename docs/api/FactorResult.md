# FactorResult

> Factor-model regression output.

| Field | Value |
|---|---|
| Group | Result types |
| Source | `src/python/result_types.rs` |
| Python import | `from qstream import FactorResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `alpha` | `float` | Estimated intercept or excess return after factor adjustment. |
| `betas` | `list[float]` | Estimated factor loadings, in input factor order. |
| `r2` | `float` | Fraction of variation explained by the model. |
| `residual_variance` | `float` | Estimated unexplained variance. |

## Formula and method

Factor-model regression output.

## Public members

- `alpha` — Property
- `betas` — Property
- `r2` — Property
- `residual_variance` — Property

[Back to the API catalog](/api/).
