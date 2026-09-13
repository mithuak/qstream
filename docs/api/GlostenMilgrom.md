# GlostenMilgrom

> Glosten-Milgrom quote/adverse-selection tracker.

| Field | Value |
|---|---|
| Group | Finance · microstructure |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import GlostenMilgrom` |

## Signature

```python
GlostenMilgrom()
```

## What it computes

Microstructure model of a market maker who updates quotes to cover adverse-selection costs from informed trading.

## Why use it

Estimate trading frictions, spread, or price impact from market observations.

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `trade_price` | `float` | Observed execution price. |
| `bid` | `float` | Current bid quote. |
| `ask` | `float` | Current ask quote. |

**Output:** `float`.

## Formula and method

Glosten-Milgrom quote/adverse-selection tracker.

```text
spread = ask - bid
price_impact = spread * lambda     (adverse-selection cost)
```

Microstructure model of a market maker who updates quotes to cover
adverse-selection costs from informed trading.

## Public members

- `price_impact` — Property
- `reset()` — Clear indicator state.
- `spread` — Property
- `update(trade_price, bid, ask)` — Consume one observation and return its current result.

[Back to the API catalog](/api/).
