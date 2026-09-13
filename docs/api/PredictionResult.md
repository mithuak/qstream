# PredictionResult

> Prediction result (LPC / lattice).

| Field | Value |
|---|---|
| Group | Result types |
| Source | `src/python/result_types.rs` |
| Python import | `from qstream import PredictionResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `prediction` | `float` | One-step prediction. |
| `coefficients` | `list[float] \| None` | Model or transform coefficients. |

## Formula and method

Prediction result (LPC / lattice).

## Public members

- `coefficients` — Property
- `prediction` — Property

[Back to the API catalog](/api/).
