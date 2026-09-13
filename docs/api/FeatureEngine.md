# FeatureEngine

> Grouped feature engine. Configure with parallel lists of names, kinds, channels, and optional numeric parameters. Each `update` call feeds every feature its channel value in one Python->Rust crossing.

| Field | Value |
|---|---|
| Group | Grouped execution |
| Source | `src/python/engine.rs` |
| Python import | `from qstream import FeatureEngine` |

## Signature

```python
FeatureEngine(names, kinds, channels, params=None)
```

## What it computes

Grouped feature engine. Configure with parallel lists of names, kinds, channels, and optional numeric parameters. Each `update` call feeds every feature its channel value in one Python->Rust crossing.

## Why use it

Update several scalar features from one OHLCV bar with one Python-to-Rust call.

## Parameters

| Parameter | Type | Default | Meaning |
|---|---|---|---|
| `names` | `list[str]` | `required` | Labels for the engine's outputs, in order. |
| `kinds` | `list[str]` | `required` | Registered scalar feature kinds, in the same order as names. |
| `channels` | `list[str]` | `required` | OHLCV or derived-return source for each feature. |
| `params` | `list[float \| None] \| None` | `None` | Optional numeric parameter for each configured feature. |

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `open` | `float` | Opening price for the current bar. |
| `high` | `float` | Highest price in the current bar. |
| `low` | `float` | Lowest price in the current bar. |
| `close` | `float` | Closing price for the current bar. |
| `volume` | `float` | Volume for the current bar. |

**Output:** `list[float | None]`.

A `None` element means no value is available yet. This commonly occurs during warmup.

## Formula and method

Grouped feature engine. Configure with parallel lists of names, kinds,
channels, and optional numeric parameters. Each `update` call feeds every
feature its channel value in one Python->Rust crossing.

## Public members

- `len` — Property
- `names` — Property
- `update(open, high, low, close, volume=0.0)` — Update all features from an OHLCV bar. The `return` channel is computed internally from successive closes. Returns values in feature order (`None` where a feature is still warming up); zip with `.names` to map back to names. Returning an ordered list (not a dict) avoids allocating Python string keys on every tick.
- `update_scalar(value)` — Convenience for close-only engines: feeds `value` to every channel.

[Back to the API catalog](/api/).
