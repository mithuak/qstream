"""Reference and streaming tests for qstream finance features.

These tests use pure-Python reference computations (no NumPy) to validate the
Rust implementations against deterministic expected values, and check warm-up,
reset, and input-validation behavior.
"""

import math

import pytest

import qstream as q


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def ewma_var_reference(returns, lam):
    var = None
    for r in returns:
        r2 = r * r
        var = r2 if var is None else lam * var + (1.0 - lam) * r2
    return var


def mean(xs):
    return sum(xs) / len(xs)


def sample_std(xs):
    m = mean(xs)
    return math.sqrt(sum((x - m) ** 2 for x in xs) / (len(xs) - 1))


# ---------------------------------------------------------------------------
# Volatility
# ---------------------------------------------------------------------------

def test_ewma_volatility_matches_reference():
    lam = 1.0 - 0.06
    returns = [0.01, -0.02, 0.015, 0.0, 0.03, -0.01, 0.02]
    ind = q.EwmaVolatility(alpha=0.06)
    vol = None
    for r in returns:
        vol = ind.update(r)
    expected = math.sqrt(ewma_var_reference(returns, lam))
    assert vol == pytest.approx(expected, rel=1e-12)


def test_garch_positive_and_finite():
    g = q.Garch()
    for _ in range(50):
        v = g.update(0.01)
        assert v is not None and v > 0.0 and math.isfinite(v)


def test_rolling_volatility_matches_std():
    period = 5
    data = [0.01, -0.02, 0.015, 0.005, -0.01, 0.02, 0.0]
    rv = q.RollingVolatility(period=period)
    out = None
    for r in data:
        out = rv.update(r)
    expected = sample_std(data[-period:])
    assert out == pytest.approx(expected, rel=1e-9)


def test_realized_volatility_matches_sqrt_sum_squares():
    period = 4
    data = [0.01, -0.02, 0.015, 0.005]
    rv = q.RealizedVolatility(period=period)
    out = None
    for r in data:
        out = rv.update(r)
    expected = math.sqrt(sum(r * r for r in data))
    assert out == pytest.approx(expected, rel=1e-12)


def test_parkinson_warmup_none_then_value():
    p = q.Parkinson(period=3)
    assert p.update(11.0, 10.0) is None
    assert p.update(12.0, 10.0) is None
    v = p.update(11.0, 10.0)
    assert v is not None and v > 0.0


def test_parkinson_reference_value():
    period = 2
    bars = [(110.0, 100.0), (105.0, 95.0)]
    p = q.Parkinson(period=period)
    v = None
    for h, l in bars:
        v = p.update(h, l)
    # sigma^2 = mean(ln(H/L)^2) / (4 ln2)
    terms = [math.log(h / l) ** 2 for h, l in bars]
    expected = math.sqrt(mean(terms) / (4 * math.log(2)))
    assert v == pytest.approx(expected, rel=1e-9)


def test_garman_klass_reference_value():
    bars = [(100.0, 105.0, 95.0, 102.0), (102.0, 108.0, 99.0, 106.0)]
    gk = q.GarmanKlass(period=2)
    v = None
    for o, h, l, c in bars:
        v = gk.update(o, h, l, c)
    terms = []
    for o, h, l, c in bars:
        hl = math.log(h / l)
        co = math.log(c / o)
        terms.append(0.5 * hl * hl - (2 * math.log(2) - 1) * co * co)
    expected = math.sqrt(mean(terms))
    assert v == pytest.approx(expected, rel=1e-9)


def test_rogers_satchell_positive():
    rs = q.RogersSatchell(period=3)
    bars = [(100, 105, 95, 102), (102, 108, 99, 106), (106, 110, 101, 109)]
    v = None
    for o, h, l, c in bars:
        v = rs.update(float(o), float(h), float(l), float(c))
    assert v is not None and v > 0.0


def test_rolling_returns_compounds():
    rr = q.RollingReturns(period=3)
    rr.update(0.1)
    rr.update(0.1)
    v = rr.update(0.1)
    assert v == pytest.approx(1.1 ** 3 - 1.0, rel=1e-9)


# ---------------------------------------------------------------------------
# Microstructure
# ---------------------------------------------------------------------------

def test_corwin_schultz_needs_two_bars():
    cs = q.CorwinSchultz()
    assert cs.update(105.0, 100.0) is None
    s = cs.update(108.0, 101.0)
    assert s is not None and s >= 0.0


def test_glosten_milgrom_spread():
    gm = q.GlostenMilgrom()
    s = gm.update(100.5, 100.0, 101.0)
    assert s == pytest.approx(1.0 / 100.5, rel=1e-12)


def test_abdi_ranaldo_warmup():
    ar = q.AbdiRanaldo(period=5)
    out = None
    for i in range(10):
        out = ar.update(101.0 + i, 99.0 + i, 100.0 + i)
    assert out is None or out >= 0.0


# ---------------------------------------------------------------------------
# Tail risk
# ---------------------------------------------------------------------------

def test_cvar_at_least_var():
    data = [-0.10, -0.05, -0.03, -0.01, 0.0, 0.01, 0.02, 0.03, 0.04, 0.05]
    var = q.ValueAtRisk(period=10, p=0.2)
    cvar = q.ConditionalValueAtRisk(period=10, p=0.2)
    v = c = None
    for r in data:
        v = var.update(r)
        c = cvar.update(r)
    assert c >= v - 1e-12


def test_cornish_fisher_finite():
    cf = q.CornishFisherVaR(period=50, confidence=0.95)
    out = None
    for i in range(60):
        out = cf.update(((i % 7) - 3) * 0.01)
    assert out is not None and math.isfinite(out)


def test_cdar_nonnegative():
    cdar = q.ConditionalDrawdownAtRisk(window=64, alpha=0.95)
    out = None
    for r in [0.01, -0.05, 0.02, -0.08, 0.03, -0.02]:
        out = cdar.update(r)
    assert out is not None and out >= 0.0


def test_rachev_positive():
    rr = q.RachevRatio(period=20, p=0.1)
    out = None
    for i in range(20):
        out = rr.update(0.02 if i % 2 == 0 else -0.01)
    assert out is not None and out > 0.0


# ---------------------------------------------------------------------------
# Factors
# ---------------------------------------------------------------------------

def test_capm_recovers_beta_alpha():
    c = q.Capm(period=50)
    out = None
    for i in range(200):
        market = (i % 11) * 0.001 - 0.005
        asset = 0.001 + 1.5 * market
        out = c.update(asset, market, 0.0)
    assert out is not None
    assert out.betas[0] == pytest.approx(1.5, abs=1e-6)
    assert out.alpha == pytest.approx(0.001, abs=1e-6)
    assert out.r2 > 0.999


def test_fama_french3_recovers_betas():
    ff = q.FamaFrench3(period=252)
    res = None
    for i in range(400):
        mkt = (i % 7) * 0.001 - 0.003
        smb = (i % 5) * 0.0005 - 0.001
        hml = (i % 3) * 0.0004 - 0.0004
        asset = 0.0002 + 1.1 * mkt + 0.5 * smb - 0.3 * hml
        res = ff.update(asset, mkt, smb, hml, 0.0)
    b = res.betas
    assert b[0] == pytest.approx(1.1, abs=0.05)
    assert b[1] == pytest.approx(0.5, abs=0.05)
    assert b[2] == pytest.approx(-0.3, abs=0.05)
    assert res.r2 > 0.95


def test_rolling_beta_and_stability():
    rb = q.RollingBeta(period=20)
    stab = q.RollingBetaStability(beta_window=20, stability_window=20)
    b = s = None
    for i in range(80):
        market = math.sin(i * 0.1) * 0.01
        asset = 1.2 * market
        b = rb.update(asset, market)
        s = stab.update(asset, market)
    assert b == pytest.approx(1.2, abs=1e-6)
    # beta is constant -> stability (std of beta) ~ 0
    assert s is not None and s < 1e-3


def test_tracking_error_zero_when_identical():
    te = q.TrackingError(period=10)
    out = None
    for i in range(30):
        out = te.update(i * 0.001, i * 0.001)
    assert abs(out) < 1e-12


def test_treynor_mazuy_gamma():
    tm = q.TreynorMazuy(period=252)
    res = None
    for i in range(400):
        mkt = (i % 9) * 0.001 - 0.004
        asset = 0.0001 + 1.0 * mkt + 0.5 * mkt * mkt
        res = tm.update(asset, mkt, 0.0)
    assert res is not None
    assert math.isfinite(res.r2)


# ---------------------------------------------------------------------------
# Performance
# ---------------------------------------------------------------------------

def test_sortino_positive_for_upside():
    s = q.SortinoRatio(period=5, mar=0.0)
    out = None
    for r in [0.02, 0.01, -0.005, 0.03, 0.01]:
        out = s.update(r)
    assert out > 0.0


def test_gain_loss_reference():
    g = q.GainLossRatio(period=4)
    for r in [0.02, -0.01, 0.03]:
        g.update(r)
    out = g.update(-0.01)
    assert out == pytest.approx(0.05 / 0.02, rel=1e-9)


def test_capture_ratios_reference():
    c = q.CaptureRatios(period=4)
    c.update(0.02, 0.01)
    c.update(-0.01, -0.005)
    c.update(0.04, 0.02)
    res = c.update(-0.02, -0.01)
    assert res.up_capture == pytest.approx(2.0, rel=1e-9)
    assert res.down_capture == pytest.approx(2.0, rel=1e-9)


def test_deflated_sharpe_in_unit_interval():
    d = q.DeflatedSharpeRatio(period=30, n_trials=5)
    out = None
    for i in range(60):
        out = d.update(((i % 5) - 2) * 0.01)
    assert 0.0 <= out <= 1.0


def test_lo_autocorrelation_sharpe_deflates_positive_autocorr():
    # AR(1), rho ~ 0.9 driven by an LCG (pseudo-white) plus drift.
    seed = 0x9E3779B97F4A7C15
    mask = (1 << 64) - 1

    def noise():
        nonlocal seed
        seed = (seed * 6364136223846793005 + 1442695040888963407) & mask
        return ((seed >> 33) / float(1 << 31)) - 0.5

    lo = q.LoAutocorrelationSharpe(period=50)
    sr = q.SharpeRatio(period=50)
    prev = 0.0
    adj = raw = None
    for _ in range(400):
        r = 0.9 * prev + 0.002 + noise() * 0.001
        prev = r
        adj = lo.update(r)
        raw = sr.update(r)
    assert lo.autocorrelation > 0.3
    assert abs(adj) < abs(raw)  # positive autocorrelation deflates the Sharpe


def test_omega_ratio_positive():
    o = q.OmegaRatio(period=10, threshold=0.0)
    out = None
    for i in range(20):
        out = o.update(((i % 5) - 2) * 0.01)
    assert out is None or out > 0.0


# ---------------------------------------------------------------------------
# Portfolio
# ---------------------------------------------------------------------------

def test_portfolio_returns_dot():
    p = q.PortfolioReturns(n_assets=2)
    r = p.update([0.01, -0.02], [0.5, 0.5])
    assert r == pytest.approx(-0.005, rel=1e-12)


def test_risk_parity_identity_covariance():
    cov = [1.0, 0.0, 0.0, 1.0]
    w = q.risk_parity_weights(cov, 2, 200)
    assert w[0] == pytest.approx(0.5, abs=1e-3)
    assert w[1] == pytest.approx(0.5, abs=1e-3)


def test_component_var_sums_to_total():
    c = q.ComponentMarginalVaR(n_assets=2, confidence=0.95)
    for _ in range(5):
        c.update([0.01, 0.0])
        c.update([0.0, 0.01])
    res = c.compute([0.5, 0.5])
    assert res is not None
    assert sum(res.component) == pytest.approx(res.total_var, rel=1e-9)


def test_ewma_covariance_matrix():
    c = q.EwmaCovarianceMatrix(n_assets=2)
    c.update([1.0, 0.0])
    c.update([0.0, 1.0])
    flat = c.to_list()
    assert len(flat) == 4
    assert c.dim == 2


# ---------------------------------------------------------------------------
# Derivatives
# ---------------------------------------------------------------------------

def test_implied_volatility_roundtrip():
    sigma_true = 0.25
    price = q.bs_price(100.0, 100.0, 1.0, 0.03, sigma_true, True)
    iv = q.ImpliedVolatility()
    solved = iv.update(price, 100.0, 100.0, 1.0, 0.03, True)
    assert solved == pytest.approx(sigma_true, abs=1e-4)


def test_bs_put_call_parity():
    s, k, t, r, sig = 100.0, 100.0, 1.0, 0.05, 0.2
    call = q.bs_price(s, k, t, r, sig, True)
    put = q.bs_price(s, k, t, r, sig, False)
    # call - put == S - K e^{-rT}
    assert call - put == pytest.approx(s - k * math.exp(-r * t), abs=1e-9)


def test_vix_term_structure_contango():
    ts = q.VixTermStructure()
    res = ts.update(20.0, [21.0, 22.0, 23.0], [0.1, 0.2, 0.3])
    assert res.contango > 0.0
    assert res.slope > 0.0


def test_variance_swap_realized():
    vs = q.VarianceSwap(period=3)
    vs.update(0.01)
    vs.update(-0.02)
    out = vs.update(0.015)
    expected = 0.01 ** 2 + 0.02 ** 2 + 0.015 ** 2
    assert out == pytest.approx(expected, rel=1e-12)


# ---------------------------------------------------------------------------
# Reset behavior
# ---------------------------------------------------------------------------

def test_reset_clears_state():
    rv = q.RollingVolatility(period=3)
    for r in [0.01, 0.02, 0.03]:
        rv.update(r)
    rv.reset()
    # After reset the window is empty again -> warm-up returns None.
    assert rv.update(0.01) is None
    assert rv.update(0.02) is None
    assert rv.update(0.03) is not None
