# StateEstimate

> State estimate from a Kalman / state-space filter.

| Field | Value |
|---|---|
| Group | Result types |
| Source | `src/python/result_types.rs` |
| Python import | `from qstream import StateEstimate` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `value` | `float` | Estimated scalar value. |
| `velocity` | `float \| None` | Estimated rate of change, when available. |
| `covariance` | `list[float]` | State uncertainty or covariance values. |

## Formula and method

State estimate from a Kalman / state-space filter.

## Public members

- `covariance` — Property
- `value` — Property
- `velocity` — Property

[Back to the API catalog](/api/).
