# Python quickstart

qstream is a Python package backed by a native Rust extension. It requires
Python 3.9 or newer and has no NumPy, pandas, SciPy, or Numba runtime dependency.
To build the extension in a local environment, run these commands from the
project root:

```bash
uv venv --python 3.14
uv pip install maturin pytest
maturin develop --release
```

Activate the environment before running Python, or use `.venv/bin/python`.
You can also build a wheel with `maturin build --release -o dist` and install
that wheel into another environment.

## Update an indicator

`EwmaVolatility` takes returns, not prices. Each call advances the same Rust
state that a historical replay would use:

```python
from qstream import EwmaVolatility

returns = [0.01, -0.005, 0.012, -0.008]
vol = EwmaVolatility(alpha=0.06)

for value in returns:
    print(vol.update(value))
```

Most streaming features return a number or `None` until enough data is
available. Check for `None` explicitly: zero can be a valid result.

## Batch the Python-to-Rust crossing

Scalar features with `update_many` accept a sequence and return one output per
input, including `None` for warmup positions. They advance the same instance,
so create a fresh one when replaying the same data for comparison.

```python
from qstream import EwmaVolatility

values = [0.01, -0.005, 0.012, -0.008]
outputs = EwmaVolatility(alpha=0.06).update_many(values)
print(outputs)
```

For different input shapes, call `update` with the fields the feature needs:

```python
from qstream import GarmanKlass, RollingBeta

ohlc_vol = GarmanKlass(period=20)
estimate = ohlc_vol.update(100.0, 102.0, 99.0, 101.0)

beta = RollingBeta(period=60)
estimate = beta.update(0.01, 0.008)  # asset return, market return
```

## Read a structured result

Factor and spectral calculations return result objects rather than plain
floats. Their fields are listed in the [API catalog](/api/).

```python
from qstream import FamaFrench3

model = FamaFrench3(period=252)
result = model.update(0.01, 0.008, 0.002, -0.001, risk_free=0.0)
if result is not None:
    print(result.alpha, result.betas, result.r2)
```

## Update several features from one bar

`FeatureEngine` accepts parallel lists: a display name, feature kind, and
input channel for each output. `return` is derived internally from successive
closes. Results stay in the order of `names`.

```python
from qstream import FeatureEngine

engine = FeatureEngine(
    names=["volatility", "regime"],
    kinds=["ewma_volatility", "page_hinkley"],
    channels=["return", "close"],
)
values = engine.update(100.0, 102.0, 99.0, 101.0, volume=1_000.0)
print(dict(zip(engine.names, values)))
```

See [streaming and warmup](/streaming) for state behavior and
[results and cadence](/results-and-cadence) for scheduled computations.
