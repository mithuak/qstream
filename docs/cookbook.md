# Cookbook

These examples use the public Python API. The source data is deliberately
small; use your own observations in a live loop or backtest.

## Compare streaming and batch values

```python
from qstream import EwmaVolatility

returns = [0.01, -0.02, 0.005, 0.012]
stream = EwmaVolatility(alpha=0.06)
live = [stream.update(value) for value in returns]
batch = EwmaVolatility(alpha=0.06).update_many(returns)
assert live == batch
```

## Keep the latest spectral result

```python
from qstream import WelchPsd

psd = WelchPsd(window=64, update_every=8)
latest = None
for sample in samples:
    result = psd.update(sample)
    if result is not None:
        latest = result
    if latest is not None:
        use_spectrum(latest)
```

`samples` and `use_spectrum` are application-provided. Retaining `latest`
lets downstream code use the most recent computation between cadence ticks.

## Label grouped outputs

```python
from qstream import FeatureEngine

engine = FeatureEngine(
    names=["vol", "change"],
    kinds=["ewma_volatility", "page_hinkley"],
    channels=["return", "close"],
)

for bar in bars:
    values = engine.update(bar.open, bar.high, bar.low, bar.close, bar.volume)
    feature_map = dict(zip(engine.names, values))
    consume(feature_map)
```

`bars` and `consume` are application-provided. Build a dictionary only if your
consumer needs named lookup; the engine itself returns an ordered list to
avoid per-tick string-key allocation.
