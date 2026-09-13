# ConditionalDrawdownAtRisk

> Conditional Drawdown-at-Risk from a rolling wealth curve.

| Field | Value |
|---|---|
| Group | Finance · tail risk |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import ConditionalDrawdownAtRisk` |

## Signature

```python
ConditionalDrawdownAtRisk(window=252, alpha=0.95)
```

## What it computes

Average of the largest drawdowns over a rolling window; a tail-risk measure of drawdown depth.

## Why use it

Track downside exposure and compare risk across return histories.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `252` | Number of recent samples retained for each calculation. |
| `alpha` | `float` | `0.95` | Confidence level for the drawdown tail estimate. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Conditional Drawdown-at-Risk from a rolling wealth curve.

```text
DD_t = peak_t - wealth_t
CDaR = E[ DD | DD >= VaR(DD) ]
```

Average of the largest drawdowns over a rolling window; a tail-risk
measure of drawdown depth.

## Public members

- `max_drawdown` — Property
- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
