# HigherOrderCumulants

> Higher-order cumulants (skewness and excess kurtosis).

| Field | Value |
|---|---|
| Group | Experimental · Higher-order spectra |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import HigherOrderCumulants` |

## Signature

```python
HigherOrderCumulants(window=64, update_every=16)
```

## What it computes

Third and fourth standardized moments of the return/signal distribution.

## Why use it

Inspect nonlinear or non-Gaussian interactions among frequencies.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `window` | `int` | `64` | Number of recent samples retained for each calculation. |
| `update_every` | `int` | `16` | Number of updates between full recalculations after the window is ready. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `PyCumulantResult | None`.

See [PyCumulantResult](/api/PyCumulantResult) for its output fields.

A `None` result means no value is available yet. It can occur during warmup or between scheduled calculations.

**Cadence:** `update_every` controls when the heavier calculation runs after samples enter the rolling window. An intervening update can return `None`.

## Formula and method

Higher-order cumulants (skewness and excess kurtosis).

```text
skewness = m3 / m2^{3/2}
kurtosis = m4 / m2^2 - 3
```

Third and fourth standardized moments of the return/signal distribution.

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
