# TermStructureResult

> VIX term-structure shape.

| Field | Value |
|---|---|
| Group | Result types |
| Source | `src/python/result_types.rs` |
| Python import | `from qstream import TermStructureResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `slope` | `float` | Term-structure slope. |
| `curvature` | `float` | Term-structure curvature. |
| `contango` | `float` | Nearest futures price relative to spot VIX, as a fractional premium or discount. |

## Formula and method

VIX term-structure shape.

## Public members

- `contango` — Property
- `curvature` — Property
- `slope` — Property

[Back to the API catalog](/api/).
