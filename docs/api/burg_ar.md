# burg_ar

> Burg AR estimation on a data window (batch helper).

| Field | Value |
|---|---|
| Group | Signal · prediction |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import burg_ar` |

## Signature

```python
burg_ar(data, order)
```

## What it computes

Returns `(ar_coefficients, reflection_coefficients)` from the forward and backward prediction errors.

## Why use it

Model short-horizon dynamics or forecast a future sample.

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `data` | `list[float]` | Observed signal samples used to fit the AR model. |
| `order` | `int` | Requested autoregressive model order. |

**Output:** `tuple[list[float], list[float]]`.

## Formula and method

Burg AR estimation on a data window (batch helper).

```text
k_m = 2 sum e_f[t] e_b[t-1] / sum (e_f[t]^2 + e_b[t-1]^2)
```

Returns `(ar_coefficients, reflection_coefficients)` from the forward and
backward prediction errors.

[Back to the API catalog](/api/).
