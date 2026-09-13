# TeagerKaiser

> Teager-Kaiser energy operator (one-step delayed).

| Field | Value |
|---|---|
| Group | Signal · regime / energy |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import TeagerKaiser` |

## Signature

```python
TeagerKaiser()
```

## What it computes

Discrete energy operator; for a sinusoid of amplitude A and frequency f it returns roughly A^2 sin^2(2 pi f).

## Why use it

Detect changes in the behavior or energy of a stream.

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Teager-Kaiser energy operator (one-step delayed).

```text
Psi[x_t] = x_{t-1}^2 - x_t x_{t-2}
```

Discrete energy operator; for a sinusoid of amplitude A and frequency f it
returns roughly A^2 sin^2(2 pi f).

## Public members

- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its result, or `None` when unavailable.
- `update_many(values)` — Batch update: process many values in a single Python->Rust crossing. Returns one output per input (`None` while warming up).

[Back to the API catalog](/api/).
