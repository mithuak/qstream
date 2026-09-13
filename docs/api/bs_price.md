# bs_price

> Black-Scholes European option price (convenience function).

| Field | Value |
|---|---|
| Group | Finance · derivatives |
| Source | `src/python/finance.rs` |
| Python import | `from qstream import bs_price` |

## Signature

```python
bs_price(spot, strike, ttm, rate, sigma, is_call=True)
```

## What it computes

Black-Scholes European option price (convenience function).

## Why use it

Monitor option or volatility-linked instrument prices and exposures.

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `spot` | `float` | Underlying spot price. |
| `strike` | `float` | Option strike price. |
| `ttm` | `float` | Time to maturity in years. |
| `rate` | `float` | Continuously compounded risk-free rate. |
| `sigma` | `float` | Volatility assumed by the option-pricing model. |
| `is_call` | `bool` | True for a call, false for a put. |

**Output:** `float`.

## Formula and method

Black-Scholes European option price (convenience function).

```text
C = S N(d1) - K e^{-rT} N(d2)
P = K e^{-rT} N(-d2) - S N(-d1)
```

[Back to the API catalog](/api/).
