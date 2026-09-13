# PyVarianceSwapResult

> Structured output returned by a qstream calculation.

| Field | Value |
|---|---|
| Group | Experimental · Result types |
| Source | `src/python/finance_experimental.rs` |
| Python import | `from qstream import PyVarianceSwapResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `strike` | `float` | Calculated variance-swap strike. |
| `realized_variance` | `float` | Variance realized over the observed window. |
| `correction` | `float` | Adjustment term in the variance-swap calculation. |

## Public members

- `correction` — Property
- `realized_variance` — Property
- `strike` — Property

[Back to the API catalog](/api/).
