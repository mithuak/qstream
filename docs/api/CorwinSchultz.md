# CorwinSchultz

> Corwin-Schultz high-low bid-ask spread estimator.

| Field | Value |
|---|---|
| Group | Finance · microstructure |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import CorwinSchultz` |

## Signature

```python
CorwinSchultz()
```

## What it computes

Bid-ask spread from the high and low prices of two consecutive periods.

## Why use it

Estimate trading frictions, spread, or price impact from market observations.

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `high` | `float` | Highest price in the current bar. |
| `low` | `float` | Lowest price in the current bar. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Corwin-Schultz high-low bid-ask spread estimator.

```text
S = 2 (e^{alpha} - 1) / (1 + e^{alpha})
alpha = sqrt(2 beta) - sqrt(beta) / (3 - 2 sqrt(2)) - sqrt(gamma / (3 - 2 sqrt(2)))
beta = E[(ln(H_t/L_t))^2], gamma = (ln(H_{t,t+1}/L_{t,t+1}))^2
```

Bid-ask spread from the high and low prices of two consecutive periods.

## Public members

- `reset()` — Clear indicator state.
- `update(high, low)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
