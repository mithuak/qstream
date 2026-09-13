# VixTermStructure

> VIX futures term-structure shape from one snapshot.

| Field | Value |
|---|---|
| Group | Finance · derivatives |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import VixTermStructure` |

## Signature

```python
VixTermStructure()
```

## What it computes

Contango/backwardation and slope of the VIX futures curve.

## Why use it

Monitor option or volatility-linked instrument prices and exposures.

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `spot_vix` | `float` | Current spot VIX level. |
| `futures` | `list[float]` | Vector of VIX futures prices. |
| `maturities` | `list[float]` | Time to maturity for each futures contract. |

**Output:** `TermStructureResult | None`.

See [TermStructureResult](/api/TermStructureResult) for its output fields.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

VIX futures term-structure shape from one snapshot.

```text
slope = (F(T2) - F(T1)) / (T2 - T1)
contango = F(T) - spot
curvature = second derivative of the futures curve
```

Contango/backwardation and slope of the VIX futures curve.

## Public members

- `update(spot_vix, futures, maturities)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
