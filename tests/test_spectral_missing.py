"""Tests for the remaining workbook estimators (spectral / filtering completion)."""

import math
import qstream as q


def sine(n, freq):
    return [math.sin(2 * math.pi * freq * i) for i in range(n)]


def test_cepstral_analysis_runs():
    c = q.CepstralAnalysis(window=64, update_every=16)
    result = None
    for x in sine(256, 0.1):
        r = c.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert len(result.frequencies) == len(result.power)
    assert all(math.isfinite(p) for p in result.power)


def test_daniell_periodogram_finds_tone():
    d = q.DaniellPeriodogram(window=64, m=3, update_every=16)
    result = None
    for x in sine(256, 0.1):
        r = d.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert result.dominant_frequency is not None
    assert abs(result.dominant_frequency - 0.1) < 0.05


def test_eigenvector_frequency_estimator_runs():
    e = q.EigenvectorFrequencyEstimator(window=64, order=4, nfft=128, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = e.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert result.dominant_frequency is not None


def test_modified_covariance_ar_finds_tone():
    m = q.ModifiedCovarianceArSpectrum(window=64, order=4, nfft=128, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = m.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert abs(result.dominant_frequency - 0.15) < 0.05


def test_multiple_coherence_identical_is_one():
    m = q.MultipleCoherence(window=64, nw=3.0, tapers=4, update_every=32)
    result = None
    s = sine(256, 0.1)
    for x in s:
        r = m.update(x, x)
        if r is not None:
            result = r
    assert result is not None
    mean = sum(result.power) / len(result.power)
    assert mean > 0.9


def test_multivariate_spectral_analysis_runs():
    m = q.MultivariateSpectralAnalysis(window=64, embedding=3, nfft=64, update_every=16)
    result = None
    for x in sine(256, 0.1):
        r = m.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert all(math.isfinite(p) for p in result.power)


def test_parzen_periodogram_finds_tone():
    p = q.ParzenPeriodogram(window=64, update_every=16)
    result = None
    for x in sine(256, 0.1):
        r = p.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert abs(result.dominant_frequency - 0.1) < 0.05


def test_partial_coherence_bounds():
    p = q.PartialCoherence(window=64, update_every=16)
    result = None
    s = sine(256, 0.1)
    for x in s:
        r = p.update(x, x)
        if r is not None:
            result = r
    assert result is not None
    assert all(0.0 <= v <= 1.0001 for v in result.power)


def test_spectral_envelope_runs():
    e = q.SpectralEnvelope(window=64, cepstral_order=8, update_every=16)
    result = None
    for x in sine(256, 0.1):
        r = e.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert all(math.isfinite(p) and p >= 0.0 for p in result.power)


def test_wiener_hopf_filter_smooths():
    w = q.WienerHopfFilter(window=64, order=4, update_every=8)
    outputs = []
    for i in range(256):
        x = math.sin(i * 0.1) + 0.5 * math.sin(i * 0.7)
        y = w.update(x)
        if y is not None:
            outputs.append(y)
    assert len(outputs) > 0
    assert all(math.isfinite(y) for y in outputs)
    assert len(w.coefficients) == 4


def test_apes_spectrum_runs():
    a = q.ApesSpectrum(window=64, order=4, nfft=128, update_every=16)
    result = None
    for x in sine(256, 0.15):
        r = a.update(x)
        if r is not None:
            result = r
    assert result is not None
    assert all(math.isfinite(p) for p in result.power)


def test_all_missing_have_pydocs():
    names = [
        "CepstralAnalysis", "DaniellPeriodogram", "EigenvectorFrequencyEstimator",
        "ModifiedCovarianceArSpectrum", "MultipleCoherence",
        "MultivariateSpectralAnalysis", "ParzenPeriodogram", "PartialCoherence",
        "SpectralEnvelope", "WienerHopfFilter", "ApesSpectrum",
    ]
    for name in names:
        assert getattr(q, name).__doc__, f"{name} missing docstring"
