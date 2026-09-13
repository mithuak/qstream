# Results and cadence

qstream exposes scalar values and structured results. Examples include
`FactorResult` (`alpha`, `betas`, `r2`, `residual_variance`), `SpectrumResult`
(`frequencies`, `power`, `dominant_frequency`, `peak_power`), and
`StateEstimate` (`value`, `velocity`, `covariance`). The
[API catalog](/api/) lists the fields on every result type.

```python
from qstream import WelchPsd, spectral_shape

psd = WelchPsd(window=512, update_every=32)
for sample in signal_samples:
    spectrum = psd.update(sample)
    if spectrum is not None:
        shape = spectral_shape(spectrum)
        print(spectrum.dominant_frequency, shape.centroid)
```

The loop above assumes `signal_samples` is your own numeric input stream.
Tier C transforms retain a rolling window but only perform the expensive
calculation on the configured `update_every` cadence. `None` on an intervening
tick means there is no new result; it does not erase the previous one. If a
downstream consumer needs the latest spectrum every tick, retain it yourself.

Result collections are created when a computation emits. Spectral outputs can
be much larger than scalar outputs, so choose a window and cadence that match
your latency budget. The [architecture guide](/architecture) explains the
Tier A/B/C split.
