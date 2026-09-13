# HilbertResult

> Analytic-signal result from a Hilbert transform: envelope amplitude, instantaneous phase, and instantaneous frequency.

| Field | Value |
|---|---|
| Group | Result types |
| Source | `src/python/result_types.rs` |
| Python import | `from qstream import HilbertResult` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `amplitude` | `float` | Instantaneous signal envelope. |
| `phase` | `float` | Instantaneous phase. |
| `frequency` | `float \| None` | Instantaneous frequency. |

## Formula and method

Analytic-signal result from a Hilbert transform: envelope amplitude,
instantaneous phase, and instantaneous frequency.

## Public members

- `amplitude` — Property
- `frequency` — Property
- `phase` — Property

[Back to the API catalog](/api/).
