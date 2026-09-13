# LoAutocorrelationSharpe

> Lo (2002) autocorrelation-adjusted Sharpe ratio.

| Field | Value |
|---|---|
| Group | Finance · performance |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import LoAutocorrelationSharpe` |

## Signature

```python
LoAutocorrelationSharpe(period=252, risk_free=0.0)
```

## What it computes

Corrects the annualized Sharpe ratio for serial correlation `rho_k` in returns.

## Why use it

Evaluate return quality relative to risk, downside, or a benchmark.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `252` | Rolling length or effective horizon; see the formula for this indicator's convention. |
| `risk_free` | `float` | `0.0` | Risk-free return used as the performance baseline. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Lo (2002) autocorrelation-adjusted Sharpe ratio.

```text
SR_adj = SR * sqrt( (1 + 2 sum_k (1 - k/N) rho_k) / N )
```

Corrects the annualized Sharpe ratio for serial correlation `rho_k` in
returns.

## Public members

- `autocorrelation` — Property
- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
