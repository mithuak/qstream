# ChangeResult

> Change-detection result.

| Field | Value |
|---|---|
| Group | Result types |
| Source | `src/python/result_types.rs` |
| Python import | `from qstream import ChangeResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `changed` | `bool` | Whether the detector signaled a change. |
| `score` | `float` | Current change-detection score. |

## Formula and method

Change-detection result.

## Public members

- `changed` — Property
- `score` — Property

[Back to the API catalog](/api/).
