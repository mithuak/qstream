---
layout: home
title: qstream documentation
titleTemplate: false

hero:
  name: qstream
  text: Finance and signal features at streaming speed
  tagline: Python-first API, Rust computation. Feed live ticks or replay history through the same update path.
  actions:
    - theme: brand
      text: Get started
      link: /quickstart
    - theme: alt
      text: Browse the API
      link: /api/
    - theme: alt
      text: Explore the workbook
      link: /workbook

features:
  - title: One update at a time
    details: Stateful indicators consume scalar, OHLC, pair, factor, or portfolio inputs. None marks warmup or a skipped scheduled computation.
  - title: Python API, Rust state
    details: Python wrappers validate inputs while the Rust core owns rolling buffers, matrix state, FFT plans, and calculations.
  - title: Work at the right cadence
    details: Small recursive updates run every tick. Heavy spectral and wavelet transforms can recompute every N ticks.
  - title: Group related features
    details: FeatureEngine updates several scalar features from one OHLCV bar in a single Python call.
---

Start with the [Python quickstart](/quickstart), then use the [API catalog](/api/)
to find constructor signatures and methods. The [workbook map](/workbook)
preserves the original planning taxonomy and links back to the source workbook.
