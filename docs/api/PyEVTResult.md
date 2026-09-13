# PyEVTResult

> Structured output returned by a qstream calculation.

| Field | Value |
|---|---|
| Group | Experimental · Result types |
| Source | `src/python/finance_experimental.rs` |
| Python import | `from qstream import PyEVTResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `var` | `float` | Estimated loss threshold at the requested tail probability. |
| `expected_shortfall` | `float` | Expected loss beyond the VaR threshold. |
| `shape` | `float` | Shape parameter of the fitted extreme-value tail. |
| `scale` | `float` | Scale parameter of the fitted extreme-value tail. |
| `n_exceedances` | `int` | Number of observations used in the tail fit. |

## Public members

- `expected_shortfall` — Property
- `n_exceedances` — Property
- `scale` — Property
- `shape` — Property
- `var` — Property

[Back to the API catalog](/api/).
