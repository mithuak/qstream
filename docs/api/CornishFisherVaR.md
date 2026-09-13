# CornishFisherVaR

> Cornish-Fisher modified VaR from rolling moments.

| Field | Value |
|---|---|
| Group | Finance · tail risk |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import CornishFisherVaR` |

## Signature

```python
CornishFisherVaR(period=252, confidence=0.95)
```

## What it computes

Adjusts the normal quantile `z` for skewness `S` and excess kurtosis `K`.

## Why use it

Track downside exposure and compare risk across return histories.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `252` | Rolling length or effective horizon; see the formula for this indicator's convention. |
| `confidence` | `float` | `0.95` | Confidence level for the reported risk estimate. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Cornish-Fisher modified VaR from rolling moments.

```text
z_cf = z + (z^2 - 1) S/6 + (z^3 - 3z) K/24 - (2z^3 - 5z) S^2/36
VaR = -(mu + sigma z_cf)
```

Adjusts the normal quantile `z` for skewness `S` and excess kurtosis `K`.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
