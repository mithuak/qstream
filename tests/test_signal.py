"""Reference and streaming tests for qstream signal-processing features."""

import math

import pytest

import qstream as q


def sine(n, freq):
    return [math.sin(2 * math.pi * freq * i) for i in range(n)]


# ---------------------------------------------------------------------------
# Filters
# ---------------------------------------------------------------------------

def test_savgol_preserves_linear():
    sg = q.SavitzkyGolay(window=5, order=2)
    out = None
    for i in range(12):
        out = sg.update(float(i))
    # center of last window [7,8,9,10,11] is 9
    assert out == pytest.approx(9.0, abs=1e-9)


def test_savgol_derivative_is_slope():
    sg = q.SavitzkyGolay(window=5, order=2, deriv=1)
    out = None
    for i in range(12):
        out = sg.update(3.0 * i + 1.0)
    assert out == pytest.approx(3.0, abs=1e-9)


def test_savgol_requires_odd_window():
    with pytest.raises(ValueError):
        q.SavitzkyGolay(window=4, order=2)


def test_kz_constant_signal():
    kz = q.KolmogorovZurbenko(window=3, passes=2)
    out = None
    for _ in range(12):
        out = kz.update(5.0)
    assert out == pytest.approx(5.0, abs=1e-9)


def test_wiener_constant_signal():
    w = q.WienerFilter(window=5, noise_window=5)
    out = None
    for _ in range(15):
        out = w.update(2.0)
    assert out == pytest.approx(2.0, abs=1e-9)


def test_adaptive_notch_locks_onto_tone():
    anf = q.AdaptiveNotchFilter(rho=0.95, mu=1e-3, freq0=0.02)
    est = None
    early = late = 0.0
    for i in range(4000):
        est = anf.update(math.sin(2 * math.pi * 0.1 * i))
        if i < 200:
            early += est.filtered ** 2
        if i > 3800:
            late += est.filtered ** 2
    assert abs(est.frequency - 0.1) < 0.02
    assert late < early  # tone suppressed once locked


# ---------------------------------------------------------------------------
# Kalman / adaptive
# ---------------------------------------------------------------------------

def test_alpha_beta_tracks_ramp():
    ab = q.AlphaBetaTracker(alpha=0.5, beta=0.1, dt=1.0)
    est = None
    for i in range(60):
        est = ab.update(float(i))
    assert abs(est.value - 59.0) < 4.0
    assert est.velocity is not None and est.velocity > 0.0


def test_kalman_tracks_ramp():
    kf = q.KalmanFilter(dt=1.0, q=1e-3, r=1.0, p0=1.0)
    est = None
    for i in range(80):
        est = kf.update(float(i))
    assert abs(est.value - 79.0) < 6.0
    assert len(est.covariance) == 4


def test_ukf_close_to_kf_on_linear():
    kf = q.KalmanFilter(dt=1.0, q=1e-2, r=0.5, p0=1.0)
    ukf = q.UnscentedKalman(dt=1.0, q=1e-2, r=0.5, p0=1.0)
    a = b = None
    for i in range(60):
        z = i * 0.5 + (i % 3) * 0.01
        a = kf.update(z).value
        b = ukf.update(z).value
    assert abs(a - b) < 1e-2


def test_ekf_runs():
    ekf = q.ExtendedKalman(dt=1.0, q=1e-3, r=1.0, p0=1.0)
    est = None
    for i in range(50):
        est = ekf.update(float(i))
    assert est.velocity is not None


def test_adaptive_and_sqrt_kalman():
    ak = q.AdaptiveKalman(q=1e-3, r=1e-2, adapt=0.05)
    sk = q.SquareRootKalman(q=1e-3, r=1e-2, s0=1.0)
    for _ in range(100):
        va = ak.update(3.0).value
        vs = sk.update(3.0).value
    assert abs(va - 3.0) < 0.5
    assert abs(vs - 3.0) < 1e-6


def test_lms_and_rls_predict():
    lms = q.LmsFilter(order=3, mu=0.01)
    rls = q.RlsFilter(order=3, lam=1.0)
    prev = [0.0, 0.0, 0.0]
    last_lms = last_rls = None
    for i in range(300):
        x = 0.6 * prev[2] + 0.2 * prev[1] + ((i % 5) - 2) * 0.01
        last_lms = lms.update(x)
        last_rls = rls.update(x)
        prev = [prev[1], prev[2], x]
    assert last_lms is not None
    assert last_rls is not None
    assert len(rls.coefficients) == 3


# ---------------------------------------------------------------------------
# Regime / energy
# ---------------------------------------------------------------------------

def test_page_hinkley_detects_shift():
    ph = q.PageHinkley(delta=0.005, threshold=1.0)
    for _ in range(50):
        ph.update(0.0)
    fired = False
    for _ in range(200):
        if ph.update(1.0).changed:
            fired = True
    assert fired


def test_teager_kaiser_positive_for_sinusoid():
    tk = q.TeagerKaiser()
    out = None
    for i in range(20):
        out = tk.update(math.sin(i * 0.5))
    assert out is not None and out > 0.0


def test_teager_kaiser_warmup():
    tk = q.TeagerKaiser()
    assert tk.update(1.0) is None
    assert tk.update(2.0) is None
    assert tk.update(3.0) is not None


def test_zero_crossing_alternating():
    zc = q.ZeroCrossingRate(window=5, threshold=0.0)
    for v in [1.0, -1.0, 1.0, -1.0]:
        zc.update(v)
    r = zc.update(1.0)
    assert r == pytest.approx(1.0, abs=1e-12)


# ---------------------------------------------------------------------------
# Spectral
# ---------------------------------------------------------------------------

def test_welch_finds_tone():
    w = q.WelchPsd(window=64, update_every=16)
    res = None
    for x in sine(256, 0.1):
        r = w.update(x)
        if r is not None:
            res = r
    assert res is not None
    assert abs(res.dominant_frequency - 0.1) < 0.03
    assert len(res.frequencies) == len(res.power)


def test_periodogram_and_fft_density():
    for cls in (q.Periodogram, q.FftSpectralDensity, q.BartlettMethod):
        est = cls(window=32, update_every=8)
        res = None
        for x in sine(128, 0.2):
            r = est.update(x)
            if r is not None:
                res = r
        assert res is not None
        assert abs(res.dominant_frequency - 0.2) < 0.08


def test_goertzel_detects_target():
    g = q.Goertzel(frequencies=[0.1, 0.3], block=64)
    res = None
    for x in sine(128, 0.1):
        r = g.update(x)
        if r is not None:
            res = r
    assert res.power[0] > res.power[1]


def test_ar_spectrum_positive():
    ar = q.ArSpectrum(window=128, order=8, nfft=64, update_every=32)
    res = None
    for x in sine(256, 0.15):
        r = ar.update(x)
        if r is not None:
            res = r
    assert res is not None
    assert all(p >= 0.0 for p in res.power)
    assert abs(res.dominant_frequency - 0.15) < 0.05


def test_multitaper_and_blackman_tukey():
    mt = q.MultitaperPsd(window=64, nw=3.0, tapers=4, update_every=32)
    bt = q.BlackmanTukey(window=128, max_lag=16, update_every=32)
    rm = rb = None
    sig = sine(256, 0.1)
    for x in sig:
        r = mt.update(x)
        if r is not None:
            rm = r
        r = bt.update(x)
        if r is not None:
            rb = r
    assert rm is not None and all(p >= 0.0 for p in rm.power)
    assert rb is not None


def test_coherence_identical_is_one():
    coh = q.Coherence(window=64, update_every=32)
    res = None
    sig = sine(256, 0.1)
    for x in sig:
        r = coh.update(x, x)
        if r is not None:
            res = r
    mean_coh = sum(res.power) / len(res.power)
    assert mean_coh > 0.9


def test_cross_spectrum_runs():
    cs = q.CrossSpectrum(window=64, update_every=32)
    res = None
    sig = sine(256, 0.1)
    for x in sig:
        r = cs.update(x, x)
        if r is not None:
            res = r
    assert res is not None and len(res.power) > 0


def test_spectral_shape_features():
    w = q.WelchPsd(window=64, update_every=32)
    res = None
    for x in sine(256, 0.1):
        r = w.update(x)
        if r is not None:
            res = r
    shape = q.spectral_shape(res)
    assert 0.0 <= shape.entropy <= 1.0 + 1e-9
    assert 0.0 <= shape.flatness <= 1.0 + 1e-9
    assert shape.centroid > 0.0


def test_stft_produces_spectrogram():
    stft = q.ShortTimeFourierTransform(window=64, hop=32)
    res = None
    for x in sine(256, 0.1):
        r = stft.update(x)
        if r is not None:
            res = r
    assert res is not None
    assert abs(res.dominant_frequency - 0.1) < 0.04
    frames, bins, vals = stft.spectrogram()
    assert frames >= 1 and bins == 33 and len(vals) == frames * bins


def test_hilbert_envelope_and_frequency():
    h = q.HilbertTransform(window=128, update_every=64)
    res = None
    for x in sine(512, 0.05):
        r = h.update(x)
        if r is not None:
            res = r
    assert res is not None
    # Unit sinusoid: envelope ~ 1, instantaneous frequency ~ 0.05.
    assert abs(res.amplitude - 1.0) < 0.15
    assert abs(res.frequency - 0.05) < 0.02


def test_instantaneous_frequency_scalar():
    ifq = q.InstantaneousFrequency(window=128, update_every=64)
    f = None
    for x in sine(512, 0.05):
        r = ifq.update(x)
        if r is not None:
            f = r
    assert f is not None and abs(f - 0.05) < 0.02


# ---------------------------------------------------------------------------
# Wavelets
# ---------------------------------------------------------------------------

def test_dwt_energy_preservation():
    # Orthogonal Haar DWT preserves energy over a full window.
    d = q.DiscreteWaveletTransform(window=8, update_every=1)
    res = None
    data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]
    for x in data:
        r = d.update(x)
        if r is not None:
            res = r
    assert res is not None and len(res.coefficients) == 8
    e_in = sum(v * v for v in data)
    e_out = sum(c * c for c in res.coefficients)
    assert e_out == pytest.approx(e_in, rel=1e-9)


def test_modwt_and_mra_and_variance():
    modwt = q.Modwt(window=16, levels=3, update_every=8)
    mra = q.MultiresolutionAnalysis(window=16, levels=3, update_every=8)
    wv = q.WaveletVariance(window=16, levels=3, update_every=8)
    rm = rr = rv = None
    for i, x in enumerate(sine(48, 0.1)):
        a = modwt.update(x)
        b = mra.update(x)
        c = wv.update(x)
        if a is not None:
            rm = a
        if b is not None:
            rr = b
        if c is not None:
            rv = c
    assert rm is not None and len(rm.coefficients) == 3
    assert rr is not None and len(rr.coefficients) == 3
    assert rv is not None and all(v >= 0.0 for v in rv.coefficients)


def test_cwt_morlet_coefficients():
    c = q.CwtMorlet(window=32, scales=[2.0, 4.0, 8.0], update_every=16)
    res = None
    for x in sine(64, 0.05):
        r = c.update(x)
        if r is not None:
            res = r
    assert res is not None and len(res.coefficients) == 3
    assert all(v >= 0.0 for v in res.coefficients)


def test_wavelet_coherence_identical_is_one():
    wc = q.WaveletCoherence(window=32, levels=3, update_every=16)
    res = None
    for x in sine(96, 0.08):
        r = wc.update(x, x)
        if r is not None:
            res = r
    assert all(abs(c - 1.0) < 1e-6 for c in res.power)


def test_wavelet_packet_expands():
    wp = q.WaveletPacket(window=8, levels=2, update_every=4)
    res = None
    for i in range(16):
        r = wp.update(float(i))
        if r is not None:
            res = r
    assert len(res.coefficients) == 8


# ---------------------------------------------------------------------------
# Prediction
# ---------------------------------------------------------------------------

def test_levinson_durbin_recovers_ar1():
    acf = [0.9 ** k for k in range(5)]
    a, e, k = q.levinson_durbin(acf, 4)
    assert a[0] == pytest.approx(0.9, abs=1e-6)
    assert all(abs(x) < 1e-6 for x in a[1:])
    assert e > 0.0


def test_burg_ar_function():
    a, k = q.burg_ar(sine(200, 0.1), 4)
    assert len(a) == 4


def test_lpc_predictor_runs():
    lpc = q.LpcPredictor(window=64, order=8, update_every=8)
    res = None
    for x in sine(200, 0.05):
        r = lpc.update(x)
        if r is not None:
            res = r
    assert res is not None
    assert math.isfinite(res.prediction)
    assert len(res.coefficients) == 8


def test_lattice_pef_whitens():
    lat = q.LatticePredictionErrorFilter(window=64, order=2, update_every=8)
    sig = sine(300, 0.05)
    sig_sq = err_sq = cnt = 0.0
    for i, x in enumerate(sig):
        e = lat.update(x)
        if e is not None and i > 120:
            sig_sq += x * x
            err_sq += e * e
            cnt += 1
    assert cnt > 0
    assert math.sqrt(err_sq / cnt) < 0.5 * math.sqrt(sig_sq / cnt)
