# levinson_durbin

> Levinson-Durbin recursion on an autocorrelation vector (batch helper).

| Field | Value |
|---|---|
| Group | Signal · prediction |
| Source | `src/python/signal.rs` |
| Python import | `from qstream import levinson_durbin` |

## Signature

```python
levinson_durbin(acf, order)
```

## What it computes

Returns `(ar_coefficients, error_variance, reflection_coefficients)`.

## Why use it

Model short-horizon dynamics or forecast a future sample.

## Inputs and output

| Input | Type | Meaning |
|---|---|---|
| `acf` | `list[float]` | Autocorrelation sequence for the recursion. |
| `order` | `int` | Requested autoregressive model order. |

**Output:** `tuple[list[float], float, list[float]]`.

## Formula and method

Levinson-Durbin recursion on an autocorrelation vector (batch helper).

```text
r_m = r_xx(m) - sum_{i=1}^{m-1} a_i r_xx(m-i)
k_m = r_m / e_{m-1}
e_m = e_{m-1} (1 - k_m^2)
```

Returns `(ar_coefficients, error_variance, reflection_coefficients)`.

[Back to the API catalog](/api/).
