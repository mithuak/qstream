# PhaseLockedLoop

> Phase-Locked Loop (PLL) frequency estimator.

| Field | Value |
|---|---|
| Group | Experimental · Cycle |
| Source | `src/python/experimental.rs` |
| Python import | `from qstream import PhaseLockedLoop` |

## Signature

```python
PhaseLockedLoop(freq0=0.1, kp=0.2, ki=0.01)
```

## What it computes

Feedback loop that locks onto the frequency and phase of a sinusoid.

## Why use it

Track a changing phase or oscillation in a stream.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `freq0` | `float` | `0.1` | Initial or target normalized frequency for the adaptive tracker. |
| `kp` | `float` | `0.2` | Proportional gain for phase correction. |
| `ki` | `float` | `0.01` | Integral gain for phase/frequency correction. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `tuple[float, float, float]`.

## Formula and method

Phase-Locked Loop (PLL) frequency estimator.

```text
e = -x * sin(phi)             (phase detector)
f <- f + kp e + ki int(e)     (PI loop filter)
phi <- phi + 2 pi f           (NCO)
```

Feedback loop that locks onto the frequency and phase of a sinusoid.

## Public members

- `is_locked()` — Report whether the phase loop currently considers itself locked.
- `reset()` — Clear indicator state.
- `update(value)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
