# PageHinkley

> Page-Hinkley change detector.

| Field | Value |
|---|---|
| Group | Signal · regime / energy |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import PageHinkley` |

## Signature

```python
PageHinkley(delta=0.005, threshold=1.0)
```

## What it computes

Cumulative-sum change detector for a drift in the mean of a stream.

## Why use it

Detect changes in the behavior or energy of a stream.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `delta` | `float` | `0.005` | Allowed mean drift before the change score accumulates. |
| `threshold` | `float` | `1.0` | Decision threshold for the detector or filter. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `value` | `float` | Next sample of the signal or time series. |

**Output:** `ChangeResult` — Whether a change was detected and its score..

See [ChangeResult](/api/ChangeResult) for its output fields.

## Formula and method

Page-Hinkley change detector.

```text
S_t = S_{t-1} + (x_t - mu_t - delta)
M_t = min_{s<=t} S_s
changed = (S_t - M_t) > threshold
```

Cumulative-sum change detector for a drift in the mean of a stream.

## Example

```python
from qstream import PageHinkley

detector = PageHinkley(delta=0.005, threshold=1.0)
for value in [0.0, 0.1, 0.0, 2.0]:
    result = detector.update(value)
    print(result.changed, result.score)
```

## Public members

- `reset()` — Clear indicator state.
- `statistic` — Property
- `update(value)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
