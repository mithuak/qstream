# EntropicVaR

> Entropic Value-at-Risk (EVaR).

| Field | Value |
|---|---|
| Group | Experimental · Finance · tail risk |
| Source | `src/python/finance_experimental.rs` |
| Python import | `from qstream import EntropicVaR` |

## Signature

```python
EntropicVaR(window=64, alpha=0.01, update_every=16)
```

## What it computes

Coherent tail-risk measure from the Chernoff bound on the loss distribution.

## Why use it

Track downside exposure and compare risk across return histories.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `alpha` | `float` | `0.01` | Tail probability used in the entropic VaR bound. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Entropic Value-at-Risk (EVaR).

```text
EVaR = inf_{z>0} ( ln E[e^{z X}] - ln(alpha) ) / z
```

Coherent tail-risk measure from the Chernoff bound on the loss
distribution.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
