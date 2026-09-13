# SpectralRiskMeasure

> Spectral Risk Measure (exponential weighting).

| Field | Value |
|---|---|
| Group | Experimental · Finance · tail risk |
| Source | `src/python/finance_experimental.rs` |
| Python import | `from qstream import SpectralRiskMeasure` |

## Signature

```python
SpectralRiskMeasure(window=64, gamma=0.05, update_every=16)
```

## What it computes

Weighted average of loss quantiles with exponentially decaying weights controlled by risk aversion `gamma`.

## Why use it

Track downside exposure and compare risk across return histories.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `gamma` | `float` | `0.05` | Risk-aversion weight in the spectral risk measure. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Spectral Risk Measure (exponential weighting).

```text
SRM = int_0^1 phi(u) q_u(X) du
phi(u) = gamma e^{-gamma(1-u)} / (1 - e^{-gamma})
```

Weighted average of loss quantiles with exponentially decaying weights
controlled by risk aversion `gamma`.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
