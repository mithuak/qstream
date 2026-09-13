# WaveletResult

> Wavelet-domain result.

| Field | Value |
|---|---|
| Group | Result types |
| Source | `src/python/result_types.rs` |
| Python import | `from qstream import WaveletResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `coefficients` | `list[float]` | Model or transform coefficients. |
| `scales` | `list[float] \| None` | Wavelet scales associated with the coefficients. |

## Formula and method

Wavelet-domain result.

## Public members

- `coefficients` — Property
- `scales` — Property

[Back to the API catalog](/api/).
