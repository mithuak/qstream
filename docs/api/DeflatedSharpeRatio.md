# DeflatedSharpeRatio

> Deflated Sharpe Ratio (Bailey & Lopez de Prado).

| Field | Value |
|---|---|
| Group | Finance · performance |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import DeflatedSharpeRatio` |

## Signature

```python
DeflatedSharpeRatio(period=252, n_trials=1, risk_free=0.0)
```

## What it computes

Adjusts the Sharpe ratio for the selection bias of trying many strategy variants (multiple-testing correction).

## Why use it

Evaluate return quality relative to risk, downside, or a benchmark.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `252` | Rolling length or effective horizon; see the formula for this indicator's convention. |
| `n_trials` | `int` | `1` | Number of selection trials accounted for in deflation. |
| `risk_free` | `float` | `0.0` | Risk-free return used as the performance baseline. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Deflated Sharpe Ratio (Bailey & Lopez de Prado).

```text
DSR = Phi( (SR - E[max SR]) / sigma(SR) )
```

Adjusts the Sharpe ratio for the selection bias of trying many strategy
variants (multiple-testing correction).

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
