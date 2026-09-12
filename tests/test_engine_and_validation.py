"""Tests for grouped execution (FeatureEngine) and input validation."""

import math

import pytest

import qstream as q


def test_feature_engine_grouped_update():
    eng = q.FeatureEngine(
        names=["vol", "garch", "ph", "tk", "sg"],
        kinds=["ewma_volatility", "garch", "page_hinkley", "teager_kaiser", "savgol"],
        channels=["return", "return", "close", "close", "close"],
    )
    assert len(eng) == 5
    assert eng.names == ["vol", "garch", "ph", "tk", "sg"]
    values = None
    px = 100.0
    for i in range(60):
        px += math.sin(i * 0.2)
        values = eng.update(px, px + 0.5, px - 0.5, px, 1000.0)
    out = dict(zip(eng.names, values))
    assert set(out.keys()) == {"vol", "garch", "ph", "tk", "sg"}
    # garch/vol on returns should be small positive; tk on prices positive.
    assert out["garch"] is not None and out["garch"] >= 0.0
    assert out["tk"] is not None


def test_feature_engine_with_params():
    eng = q.FeatureEngine(
        names=["rv", "sharpe"],
        kinds=["realized_volatility", "sharpe"],
        channels=["return", "return"],
        params=[10.0, 30.0],
    )
    out = None
    for i in range(40):
        out = eng.update(100.0, 100.0, 100.0, 100.0, 0.0)
    assert out is not None


def test_feature_engine_new_kinds():
    eng = q.FeatureEngine(
        names=["lo", "notch"],
        kinds=["lo_sharpe", "adaptive_notch"],
        channels=["return", "close"],
    )
    values = None
    for i in range(200):
        px = 100.0 + math.sin(i * 0.2) * 2
        values = eng.update(px, px + 0.5, px - 0.5, px, 0.0)
    out = dict(zip(eng.names, values))
    # Both features should produce finite values after warm-up.
    assert out["notch"] is not None and math.isfinite(out["notch"])
    assert out["lo"] is None or math.isfinite(out["lo"])


def test_feature_engine_rejects_bad_kind():
    with pytest.raises(ValueError):
        q.FeatureEngine(
            names=["x"], kinds=["not_a_real_feature"], channels=["close"]
        )


def test_feature_engine_rejects_bad_channel():
    with pytest.raises(ValueError):
        q.FeatureEngine(names=["x"], kinds=["garch"], channels=["not_a_channel"])


def test_feature_engine_length_mismatch():
    with pytest.raises(ValueError):
        q.FeatureEngine(names=["a", "b"], kinds=["garch"], channels=["close"])


def test_feature_engine_scalar_update():
    eng = q.FeatureEngine(
        names=["tk"], kinds=["teager_kaiser"], channels=["close"]
    )
    values = None
    for i in range(10):
        values = eng.update_scalar(math.sin(i * 0.3) * 10 + 100)
    assert values is not None and len(values) == 1
    assert dict(zip(eng.names, values))["tk"] is not None


# ---------------------------------------------------------------------------
# Input validation (NaN / Inf / bad params)
# ---------------------------------------------------------------------------

@pytest.mark.parametrize(
    "factory,arg",
    [
        (lambda: q.EwmaVolatility(), 0.01),
        (lambda: q.Garch(), 0.01),
        (lambda: q.RollingVolatility(period=3), 0.01),
        (lambda: q.RealizedVolatility(period=3), 0.01),
        (lambda: q.SharpeRatio(period=3), 0.01),
        (lambda: q.SortinoRatio(period=3), 0.01),
        (lambda: q.ValueAtRisk(period=3), 0.01),
        (lambda: q.PageHinkley(), 0.01),
        (lambda: q.TeagerKaiser(), 0.01),
        (lambda: q.LmsFilter(order=2), 0.01),
        (lambda: q.RlsFilter(order=2), 0.01),
        (lambda: q.KalmanFilter(), 0.01),
        (lambda: q.AlphaBetaTracker(), 0.01),
    ],
)
def test_nan_rejected_scalar(factory, arg):
    ind = factory()
    # Warm up with a valid value first where needed.
    ind.update(arg)
    with pytest.raises(ValueError):
        ind.update(float("nan"))
    with pytest.raises(ValueError):
        ind.update(float("inf"))


def test_nan_rejected_ohlc():
    gk = q.GarmanKlass(period=3)
    with pytest.raises(ValueError):
        gk.update(100.0, float("nan"), 95.0, 100.0)


def test_nan_rejected_pair():
    cs = q.CorwinSchultz()
    with pytest.raises(ValueError):
        cs.update(float("nan"), 100.0)


def test_nan_rejected_slice():
    m = q.MultiFactorModel(n_factors=2)
    with pytest.raises(ValueError):
        m.update(0.01, [float("nan"), 0.0])


def test_positive_price_required():
    gk = q.GarmanKlass(period=3)
    with pytest.raises(ValueError):
        gk.update(-1.0, 105.0, 95.0, 100.0)


def test_bad_constructor_params():
    with pytest.raises(ValueError):
        q.RollingVolatility(period=0)
    with pytest.raises(ValueError):
        q.EwmaVolatility(alpha=1.5)
    with pytest.raises(ValueError):
        q.ValueAtRisk(period=10, p=1.5)
    with pytest.raises(ValueError):
        q.Goertzel(frequencies=[], block=8)
    with pytest.raises(ValueError):
        q.KalmanFilter(dt=0.0)


# ---------------------------------------------------------------------------
# Batch update_many (single Python->Rust crossing over a buffer)
# ---------------------------------------------------------------------------

def test_update_many_matches_sequential():
    data = [math.sin(i * 0.13) * 0.02 + ((i % 7) - 3) * 0.001 for i in range(200)]
    a = q.EwmaVolatility(alpha=0.06)
    b = q.EwmaVolatility(alpha=0.06)
    batch = a.update_many(data)
    seq = [b.update(x) for x in data]
    assert len(batch) == len(data)
    for x, y in zip(batch, seq):
        assert (x is None and y is None) or (x == pytest.approx(y, rel=1e-12, abs=1e-15))


def test_update_many_warmup_nones():
    rv = q.RollingVolatility(period=5)
    out = rv.update_many([0.01, 0.02, 0.03, 0.04, 0.05, 0.06])
    # First 4 are warm-up (None), last two are computed.
    assert out[:4] == [None, None, None, None]
    assert out[4] is not None and out[5] is not None


def test_update_many_present_on_scalar_indicators():
    for obj in [
        q.Garch(), q.RealizedVolatility(period=5), q.SharpeRatio(period=5),
        q.SortinoRatio(period=5), q.ValueAtRisk(period=5),
        q.SavitzkyGolay(window=5, order=2), q.TeagerKaiser(),
        q.LmsFilter(order=3), q.RlsFilter(order=3), q.WienerFilter(),
    ]:
        assert hasattr(obj, "update_many"), f"{type(obj).__name__} missing update_many"
        res = obj.update_many([0.01, -0.02, 0.015, 0.0, 0.03, -0.01])
        assert isinstance(res, list) and len(res) == 6


def test_update_many_rejects_nan():
    g = q.Garch()
    with pytest.raises(ValueError):
        g.update_many([0.01, float("nan"), 0.02])


# ---------------------------------------------------------------------------
# Long-running stability
# ---------------------------------------------------------------------------

def test_long_run_stability():
    ewma = q.EwmaVolatility(alpha=0.06)
    garch = q.Garch()
    rv = q.RollingVolatility(period=20)
    ph = q.PageHinkley()
    for i in range(20000):
        r = math.sin(i * 0.01) * 0.02 + ((i % 13) - 6) * 0.001
        v1 = ewma.update(r)
        v2 = garch.update(r)
        v3 = rv.update(r)
        ph.update(i * 1.0)
        assert math.isfinite(v1) and v1 >= 0.0
        assert math.isfinite(v2) and v2 >= 0.0
        assert v3 is None or (math.isfinite(v3) and v3 >= 0.0)


def test_welch_long_run_stable():
    w = q.WelchPsd(window=128, update_every=64)
    res = None
    for i in range(2000):
        r = w.update(math.sin(i * 0.05) + 0.01 * (i % 7))
        if r is not None:
            res = r
    assert res is not None
    assert all(math.isfinite(p) and p >= 0.0 for p in res.power)
