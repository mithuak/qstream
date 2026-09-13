# Streaming and warmup

Each indicator owns its state. `update` consumes one observation and advances
that state once. A historical backtest can replay the same sequence through
the same method used for live data.

```python
from qstream import RollingVolatility

returns = [0.01, -0.02, 0.005, 0.015]
stream = RollingVolatility(period=3)
one_by_one = [stream.update(r) for r in returns]

batch = RollingVolatility(period=3).update_many(returns)
assert one_by_one == batch
```

`None` can mean insufficient observations. For Tier C features, it can also
mean the feature has data but its next scheduled calculation has not occurred.
Do not treat `None` as zero, or assume every feature has the same warmup length.
Read the individual [API page](/api/) and check a short input sequence when the
exact first-emission tick matters.

Most standalone indicators expose `reset()` to clear their state. To compare
two histories, use separate instances or reset before replaying. `update_many`
is available on scalar streaming classes; it returns a list with the same
length as its input, including warmup positions. OHLC, pair, and structured
features generally use `update` with their specific argument shape.

The Python wrappers reject non-finite scalar inputs with `ValueError`.
Parameters such as periods and smoothing constants are validated at
construction. Keep input units consistent: `EwmaVolatility` and
`RealizedVolatility` consume returns, while `GarmanKlass` consumes OHLC prices.

For latency-sensitive multi-feature pipelines, [FeatureEngine](/api/FeatureEngine)
updates several supported scalar features with one Python-to-Rust call. It
accepts `open`, `high`, `low`, `close`, and optional `volume` channels, plus a
derived simple-return channel. Engine output is an ordered list, so pair it
with `engine.names` only when a name lookup is needed.
