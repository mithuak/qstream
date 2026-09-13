# RachevRatio

> Rachev ratio (upper-tail reward vs lower-tail risk).

| Field | Value |
|---|---|
| Group | Finance · tail risk |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import RachevRatio` |

## Signature

```python
RachevRatio(period=252, p=0.05)
```

## What it computes

Expected tail gain divided by expected tail loss (reward-to-risk using both distribution tails).

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

Rachev ratio (upper-tail reward vs lower-tail risk).

```text
Rachev = CVaR_p( -r ) / CVaR_p( r )
```

Expected tail gain divided by expected tail loss (reward-to-risk using
both distribution tails).

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
