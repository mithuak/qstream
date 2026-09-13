# ImpliedVolatility

> Black-Scholes implied volatility (Newton-Raphson with bisection safeguard).

| Field | Value |
|---|---|
| Group | Finance · derivatives |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import ImpliedVolatility` |

## Signature

```python
ImpliedVolatility()
```

## What it computes

Backs out the volatility implied by an observed option price.

## Why use it

Monitor option or volatility-linked instrument prices and exposures.

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `market_price` | `float` | Observed market price of the option. |
| `spot` | `float` | Underlying spot price. |
| `strike` | `float` | Option strike price. |
| `ttm` | `float` | Time to maturity in years. |
| `rate` | `float` | Continuously compounded risk-free rate. |
| `is_call` | `bool` | True for a call, false for a put. |

**Output:** `float | None`.

A `None` result means no value is available yet. This commonly occurs during warmup.

## Formula and method

Black-Scholes implied volatility (Newton-Raphson with bisection safeguard).

```text
sigma* = root of  BS(sigma) - market_price = 0
BS(sigma) = spot N(d1) - K e^{-rT} N(d2)
```

Backs out the volatility implied by an observed option price.

## Public members

- `reset()` — Clear indicator state.
- `update(market_price, spot, strike, ttm, rate=0.0, is_call=True)` — Consume one observation and return its result, or `None` when unavailable.

[Back to the API catalog](/api/).
