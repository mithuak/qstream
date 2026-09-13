# VarianceSwap

> Realized-variance tracker for variance swaps.

| Field | Value |
|---|---|
| Group | Finance · derivatives |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import VarianceSwap` |

## Signature

```python
VarianceSwap(period)
```

## What it computes

Tracks realized variance for a variance-swap payoff.

## Why use it

Monitor option or volatility-linked instrument prices and exposures.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `period` | `int` | `required` | Rolling length or effective horizon; see the formula for this indicator's convention. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next return or scalar financial observation; check the formula for the required unit. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Realized-variance tracker for variance swaps.

```text
RV = sum_t r_t^2
pnl = notional * (RV - strike_variance)
```

Tracks realized variance for a variance-swap payoff.

## Public members

- `pnl(strike_variance, notional=1.0)` — Calculate profit or loss from the current variance estimate.
- `realized_variance` — Property
- `realized_volatility` — Property
- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
