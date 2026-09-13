# ConditionalValueAtRisk

> Conditional VaR / Expected Shortfall over a rolling window.

| Field | Value |
|---|---|
| Group | Finance · tail risk |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import ConditionalValueAtRisk` |

## Signature

```python
ConditionalValueAtRisk(period=252, p=0.05)
```

## What it computes

Mean of losses beyond VaR; a coherent tail-risk measure.

## Why use it

Track downside exposure and compare risk across return histories.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `252` | Rolling length or effective horizon; see the formula for this indicator's convention. |
| `p` | `float` | `0.05` | Tail probability or probability threshold; see the indicator formula. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Conditional VaR / Expected Shortfall over a rolling window.

```text
CVaR_p = -E[ r | r <= -VaR_p ]
```

Mean of losses beyond VaR; a coherent tail-risk measure.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
