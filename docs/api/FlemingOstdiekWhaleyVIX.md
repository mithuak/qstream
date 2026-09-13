# FlemingOstdiekWhaleyVIX

> Fleming-Ostdiek-Whaley VIX Implied Volatility.

| Field | Value |
|---|---|
| Group | Experimental · Finance · volatility derivatives |
| Source | `src/python/finance_experimental.rs` |
| Python import | `from qstream import FlemingOstdiekWhaleyVIX` |

## Signature

```python
FlemingOstdiekWhaleyVIX(window=22, annualization=252.0, update_every=5)
```

## What it computes

Model-free VIX-style implied volatility index (simplified to realized variance as a proxy).

## Why use it

Analyze volatility index or variance-swap quantities.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `22` | Number of recent samples retained for each calculation. |
| `annualization` | `float` | `252.0` | Scale factor used to annualize the result. |
| `update_every` | `int` | `5` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Fleming-Ostdiek-Whaley VIX Implied Volatility.

```text
sigma^2 = (2/T) sum_K DeltaK/K^2 Q(K) - (1/T)(F/K0 - 1)^2
```

Model-free VIX-style implied volatility index (simplified to realized
variance as a proxy).

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
