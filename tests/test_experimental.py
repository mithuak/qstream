"""Tests for Phase 7 experimental features (heavy/experimental algorithms)."""

import math
import qstream as q


def sine(n, freq):
    return [math.sin(2 * math.pi * freq * i) for i in range(n)]


def two_tones(n, f1, f2):
    return [math.sin(2 * math.pi * f1 * i) + math.sin(2 * math.pi * f2 * i) for i in range(n)]


# ---------------------------------------------------------------------------
# Spectral heavy methods
# ---------------------------------------------------------------------------

def test_music_spectrum_runs():
    m = q.MusicSpectrum(64, 1, 128, 16)
    result = None
    for x in sine(256, 0.15):
        r = m.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert len(result.frequencies) > 0
    assert len(result.power) > 0


def test_esprit_spectrum_runs():
    e = q.EspritSpectrum(64, 1, 128, 16)
    result = None
    for x in sine(256, 0.2):
        r = e.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_matrix_pencil_runs():
    mp = q.MatrixPencilSpectrum(64, 1, 128, 16)
    result = None
    for x in sine(256, 0.12):
        r = mp.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_capon_runs():
    c = q.CaponSpectrum(window=64, ar_order=16, nfft=128, update_every=16)
    result = None
    for x in sine(256, 0.18):
        r = c.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_pisarenko_runs():
    p = q.PisarenkoSpectrum(64, 1, 128, 16)
    result = None
    for x in sine(256, 0.25):
        r = p.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_minimum_norm_runs():
    mn = q.MinimumNormSpectrum(64, 1, 128, 16)
    result = None
    for x in sine(256, 0.2):
        r = mn.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_prony_runs():
    p = q.PronySpectrum(64, 1, 128, 16)
    result = None
    for x in sine(256, 0.15):
        r = p.update(x)
        if r is not None:
            result = r
    assert result is not None


# ---------------------------------------------------------------------------
# Decomposition
# ---------------------------------------------------------------------------

def test_vmd_decomposition_runs():
    vmd = q.VmdDecomposition(window=64, n_modes=3, alpha=2000.0, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = vmd.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert result.n_modes > 0
    assert len(result.mode_frequencies) == result.n_modes


def test_emd_decomposition_runs():
    emd = q.EmdDecomposition(window=64, max_imfs=4, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = emd.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert result.n_modes > 0


def test_lmd_decomposition_runs():
    lmd = q.LmdDecomposition(window=64, max_pfs=4, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = lmd.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert result.n_modes > 0


def test_matching_pursuit_runs():
    mp = q.MatchingPursuitDecomposition(window=64, n_atoms=5, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = mp.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert result.n_modes > 0


def test_synchrosqueezing_runs():
    sq = q.SynchrosqueezingTransform(window=64, nfft=128, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = sq.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert all(math.isfinite(p) for p in result.power)


# ---------------------------------------------------------------------------
# Higher-order spectra
# ---------------------------------------------------------------------------

def test_bispectrum_runs():
    bs = q.BispectrumAnalysis(window=64, nfft=64, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = bs.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert len(result.values) > 0
    assert result.entropy >= 0.0


def test_bicoherence_runs():
    bc = q.BicoherenceAnalysis(window=64, nfft=64, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = bc.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert 0.0 <= result.peak <= 1.0001


def test_higher_order_cumulants_runs():
    hoc = q.HigherOrderCumulants(window=64, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = hoc.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert math.isfinite(result.skewness)
    assert math.isfinite(result.kurtosis)


# ---------------------------------------------------------------------------
# Time-frequency
# ---------------------------------------------------------------------------

def test_stockwell_runs():
    st = q.StockwellTransform(window=64, nfft=64, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = st.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_reassigned_spectrogram_runs():
    rs = q.ReassignedSpectrogram(window=64, nfft=64, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = rs.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_constant_q_runs():
    cqt = q.ConstantQTransform(window=64, n_bins=24, f_min=0.01, f_max=0.4, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = cqt.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_fractional_fourier_runs():
    frft = q.FractionalFourierTransform(window=64, angle=math.pi / 4, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = frft.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_wigner_ville_runs():
    wv = q.WignerVilleDistribution(window=64, nfft=64, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = wv.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_chirp_z_runs():
    czt = q.ChirpZTransform(window=64, f_start=0.1, f_end=0.3, n_points=64, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = czt.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_kurtogram_runs():
    kg = q.KurtogramAnalysis(window=64, nfft=64, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = kg.update(x)
        if r is not None:
            result = r
    assert result is not None


# ---------------------------------------------------------------------------
# Wavelet extras
# ---------------------------------------------------------------------------

def test_dual_tree_cwt_runs():
    dtcwt = q.DualTreeCwt(window=64, levels=3, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = dtcwt.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert len(result.coefficients) > 0


def test_empirical_wavelet_runs():
    ewt = q.EmpiricalWaveletTransform(window=64, n_modes=3, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = ewt.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_tunable_q_runs():
    tqwt = q.TunableQWavelet(window=64, levels=4, q_factor=2.0, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = tqwt.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_sure_shrink_runs():
    ss = q.SureShrinkDenoise(window=64, levels=3, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = ss.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert len(result.coefficients) == 64


def test_stationary_wavelet_runs():
    swd = q.StationaryWaveletDenoise(window=64, levels=3, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = swd.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert len(result.coefficients) == 64


def test_cross_wavelet_runs():
    xwt = q.CrossWaveletTransform(window=64, levels=3, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = xwt.update(x, x)
        if r is not None:
            result = r
    assert result is not None


def test_wavelet_phase_synchrony_runs():
    wps = q.WaveletPhaseSynchrony(window=64, levels=3, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = wps.update(x, x)
        if r is not None:
            result = r
    assert result is not None


def test_wavelet_regression_runs():
    wr = q.WaveletRegression(window=64, levels=3, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = wr.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert len(result.coefficients) == 64


# ---------------------------------------------------------------------------
# Heavy Kalman filters
# ---------------------------------------------------------------------------

def test_particle_filter_runs():
    pf = q.ParticleFilter(n_particles=200, process_noise=0.01, measurement_noise=0.1)
    result = None
    for i in range(100):
        measurement = 1.0 + (i % 7) * 0.01
        result = pf.update(measurement)
    assert result is not None
    assert math.isfinite(result.value)


def test_ensemble_kalman_runs():
    enkf = q.EnsembleKalman(n_members=50, state_dim=2, process_noise=0.01, measurement_noise=0.1)
    result = None
    for i in range(100):
        measurement = math.sin(i * 0.01) + (i % 7) * 0.01
        result = enkf.update(measurement)
    assert result is not None
    assert math.isfinite(result.value)


def test_cubature_kalman_runs():
    ckf = q.CubatureKalman(state_dim=2, process_noise=0.01, measurement_noise=0.1)
    result = None
    for i in range(100):
        measurement = math.sin(i * 0.01) + (i % 7) * 0.01
        result = ckf.update(measurement)
    assert result is not None
    assert math.isfinite(result.value)


# ---------------------------------------------------------------------------
# Cycle: PLL
# ---------------------------------------------------------------------------

def test_pll_runs():
    pll = q.PhaseLockedLoop(freq0=0.1, kp=0.2, ki=0.01)
    freq = 0.0
    for i in range(500):
        x = math.sin(2 * math.pi * 0.15 * i)
        freq, i_comp, q_comp = pll.update(x)
    assert math.isfinite(freq)


# ---------------------------------------------------------------------------
# FastICA
# ---------------------------------------------------------------------------

def test_fast_ica_runs():
    ica = q.FastICA(n_signals=2, n_components=2, window=64, update_every=16)
    result = None
    for i in range(256):
        s1 = math.sin(2 * math.pi * 0.1 * i)
        s2 = math.cos(2 * math.pi * 0.2 * i)
        mixed = [0.5 * s1 + 0.5 * s2, 0.3 * s1 + 0.7 * s2]
        r = ica.update(mixed)
        if r is not None:
            result = r
    assert result is not None
    assert len(result) == 2


# ---------------------------------------------------------------------------
# Finance: Tail risk
# ---------------------------------------------------------------------------

def test_entropic_var_runs():
    evar = q.EntropicVaR(window=64, alpha=0.01, update_every=16)
    result = None
    for i in range(256):
        x = math.sin(i * 0.1) + (i % 7) * 0.01
        r = evar.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_evt_gpd_runs():
    evt = q.EVTGpdTailRisk(window=128, alpha=0.01, threshold_quantile=0.95, update_every=32)
    result = None
    for i in range(512):
        x = math.sin(i * 0.1) + (i % 7) * 0.01
        r = evt.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert math.isfinite(result.var)
    assert math.isfinite(result.expected_shortfall)


def test_johnson_su_var_runs():
    jsu = q.JohnsonSUVaR(window=64, alpha=0.01, update_every=16)
    result = None
    for i in range(256):
        x = math.sin(i * 0.1) + (i % 7) * 0.01
        r = jsu.update(x)
        if r is not None:
            result = r
    assert result is not None


def test_spectral_risk_runs():
    srm = q.SpectralRiskMeasure(window=64, gamma=0.05, update_every=16)
    result = None
    for i in range(256):
        x = math.sin(i * 0.1) + (i % 7) * 0.01
        r = srm.update(x)
        if r is not None:
            result = r
    assert result is not None


# ---------------------------------------------------------------------------
# Finance: Portfolio optimizers
# ---------------------------------------------------------------------------

def test_max_diversification_runs():
    md = q.MaxDiversification(n_assets=3, period=32, update_every=8)
    result = None
    for i in range(128):
        returns = [
            math.sin(i * 0.1) * 0.01,
            math.cos(i * 0.15) * 0.01,
            math.sin(i * 0.05) * 0.01,
        ]
        r = md.update(returns)
        if r is not None:
            result = r
    assert result is not None
    assert len(result) == 3
    assert abs(sum(result) - 1.0) < 1e-6


def test_ewp_runs():
    ewp = q.ExponentiallyWeightedPortfolio(3, 0.94)
    result = None
    for i in range(128):
        returns = [
            math.sin(i * 0.1) * 0.01,
            math.cos(i * 0.15) * 0.01,
            math.sin(i * 0.05) * 0.01,
        ]
        r = ewp.update(returns)
        if r is not None:
            result = r
    assert result is not None
    assert len(result) == 3


# ---------------------------------------------------------------------------
# Finance: Realized volatility
# ---------------------------------------------------------------------------

def test_realized_kernel_runs():
    rk = q.RealizedKernel(window=64, n_lags=10, annualization=252.0, update_every=16)
    result = None
    for i in range(256):
        r = math.sin(i * 0.05) * 0.01
        v = rk.update(r)
        if v is not None:
            result = v
    assert result is not None
    assert result >= 0.0


def test_tsrv_runs():
    tsrv = q.TwoScaleRealizedVariance(window=64, n_subsamples=5, annualization=252.0, update_every=16)
    result = None
    for i in range(256):
        r = math.sin(i * 0.05) * 0.01
        v = tsrv.update(r)
        if v is not None:
            result = v
    assert result is not None
    assert result >= 0.0


# ---------------------------------------------------------------------------
# Finance: Volatility derivatives
# ---------------------------------------------------------------------------

def test_fow_vix_runs():
    vix = q.FlemingOstdiekWhaleyVIX(window=22, annualization=252.0, update_every=5)
    result = None
    for i in range(100):
        r = math.sin(i * 0.05) * 0.01
        v = vix.update(r)
        if v is not None:
            result = v
    assert result is not None
    assert result >= 0.0


def test_vandermeer_vix_runs():
    vix = q.VandermeerVIX(window=22, annualization=252.0, update_every=5)
    result = None
    for i in range(100):
        r = math.sin(i * 0.05) * 0.01
        v = vix.update(r)
        if v is not None:
            result = v
    assert result is not None
    assert result >= 0.0


def test_demeterfi_runs():
    vs = q.DemeterfiVarianceSwap(window=22, period=0.082, annualization=252.0, update_every=5)
    result = None
    for i in range(100):
        r = math.sin(i * 0.05) * 0.01
        v = vs.update(r)
        if v is not None:
            result = v
    assert result is not None
    assert math.isfinite(result.strike)
