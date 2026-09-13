# FrequencyEstimate

> Frequency estimate from an adaptive cycle filter (e.g. adaptive notch).

| Field | Value |
|---|---|
| Group | Result types |
| Source | `src/python/result_types.rs` |
| Python import | `from qstream import FrequencyEstimate` |

Result objects are returned by feature updates; they are not constructed directly.

## Output fields

| Field | Type | Meaning |
|---|---|---|
| `frequency` | `float` | Instantaneous frequency. |
| `filtered` | `float \| None` | Signal after adaptive filtering. |

## Formula and method

Frequency estimate from an adaptive cycle filter (e.g. adaptive notch).

## Public members

- `filtered` — Property
- `frequency` — Property

[Back to the API catalog](/api/).
