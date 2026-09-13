# ComponentVaRResult

> Component / marginal VaR result.

| Field | Value |
|---|---|
| Group | Result types |
| Source | `src/python/result_types.rs` |
| Python import | `from qstream import ComponentVaRResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `total_var` | `float` | Portfolio-level value at risk. |
| `marginal` | `list[float]` | Marginal risk contribution for each asset. |
| `component` | `list[float]` | Weighted risk contribution for each asset. |

## Formula and method

Component / marginal VaR result.

## Public members

- `component` — Property
- `marginal` — Property
- `total_var` — Property

[Back to the API catalog](/api/).
