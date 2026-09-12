//! PyO3 wrappers for the remaining spectral / filtering estimators.
//! Doc comments carry the underlying formulas and become Python docstrings.

use pyo3::prelude::*;

use crate::python::result_types::SpectrumResult;
use crate::python::{chk, chk2, chk_pos};
use crate::signal::spectral_missing::{
    ApesSpectrum as CoreApes, CepstralAnalysis as CoreCepstral,
    DaniellPeriodogram as CoreDaniell, EigenvectorFrequencyEstimator as CoreEigFreq,
    ModifiedCovarianceArSpectrum as CoreModCov, MultipleCoherence as CoreMultiCoh,
    MultivariateSpectralAnalysis as CoreMultiSpec, ParzenPeriodogram as CoreParzen,
    PartialCoherence as CorePartialCoh, SpectralEnvelope as CoreSpecEnv,
    WienerHopfFilter as CoreWienerHopf,
};

/// Cepstral Analysis.
///
/// Real cepstrum: inverse Fourier transform of the log-magnitude spectrum.
///
/// ```text
/// c[n] = IFFT( log |FFT(x[n])| )
/// ```
///
/// `frequencies` are quefrency bins; `power` holds cepstral coefficients.
/// A periodic signal produces a peak at its fundamental period.
#[pyclass(module = "qstream")]
pub struct CepstralAnalysis {
    inner: CoreCepstral,
}

#[pymethods]
impl CepstralAnalysis {
    #[new]
    #[pyo3(signature = (window = 64, update_every = 16))]
    fn new(window: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(update_every, "update_every")?;
        Ok(Self { inner: CoreCepstral::new(window, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Daniell-Smoothed Periodogram.
///
/// Raw periodogram smoothed by a symmetric moving average of half-width `m`:
///
/// ```text
/// P_D(f_k) = (1 / (2m + 1)) * sum_{j = -m}^{m} P(f_{k+j})
/// ```
#[pyclass(module = "qstream")]
pub struct DaniellPeriodogram {
    inner: CoreDaniell,
}

#[pymethods]
impl DaniellPeriodogram {
    #[new]
    #[pyo3(signature = (window = 64, m = 3, update_every = 16))]
    fn new(window: usize, m: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(m, "m")?;
        Ok(Self { inner: CoreDaniell::new(window, m, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Eigenvector Frequency Estimator.
///
/// Pseudospectrum from the minimum-eigenvalue eigenvector `v` of the
/// autocorrelation matrix:
///
/// ```text
/// A(f) = sum_i v_i e^{-j 2 pi f i}
/// P(f) = 1 / |A(f)|^2
/// ```
#[pyclass(module = "qstream")]
pub struct EigenvectorFrequencyEstimator {
    inner: CoreEigFreq,
}

#[pymethods]
impl EigenvectorFrequencyEstimator {
    #[new]
    #[pyo3(signature = (window = 64, order = 4, nfft = 128, update_every = 16))]
    fn new(window: usize, order: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(order, "order")?;
        Ok(Self { inner: CoreEigFreq::new(window, order, nfft, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Modified-Covariance (Forward-Backward) AR Spectrum.
///
/// AR coefficients from minimizing the average forward/backward prediction
/// error:
///
/// ```text
/// e_f(t) = x_t + sum_i a_i x_{t-i}
/// e_b(t) = x_{t-p} + sum_i a_i x_{t-p+i}
/// P(f) = sigma^2 / |1 + sum a_i e^{-j2pi f i}|^2
/// ```
#[pyclass(module = "qstream")]
pub struct ModifiedCovarianceArSpectrum {
    inner: CoreModCov,
}

#[pymethods]
impl ModifiedCovarianceArSpectrum {
    #[new]
    #[pyo3(signature = (window = 64, order = 4, nfft = 128, update_every = 16))]
    fn new(window: usize, order: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(order, "order")?;
        Ok(Self { inner: CoreModCov::new(window, order, nfft, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Multiple (Multitaper) Coherence.
///
/// Magnitude-squared coherence averaged over DPSS eigenspectra:
///
/// ```text
/// gamma^2(f) = |sum_k S_xy^(k)(f)|^2
///              / (sum_k S_xx^(k)(f) * sum_k S_yy^(k)(f))
/// ```
///
/// Values are in `[0, 1]`.
#[pyclass(module = "qstream")]
pub struct MultipleCoherence {
    inner: CoreMultiCoh,
}

#[pymethods]
impl MultipleCoherence {
    #[new]
    #[pyo3(signature = (window = 64, nw = 3.0, tapers = 4, update_every = 32))]
    fn new(window: usize, nw: f64, tapers: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(tapers, "tapers")?;
        chk(nw)?;
        Ok(Self { inner: CoreMultiCoh::new(window, nw, tapers, update_every) })
    }
    fn update(&mut self, x: f64, y: f64) -> PyResult<Option<SpectrumResult>> {
        chk2(x, y)?;
        Ok(self.inner.update(x, y).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Multivariate Spectral Analysis.
///
/// Total power spectrum from time-delay embedding, as the trace of the
/// cross-spectral matrix:
///
/// ```text
/// S(f) = sum_{i,j} S_ij(f),  S_ij(f) = E[X_i(f) X_j*(f)]
/// ```
#[pyclass(module = "qstream")]
pub struct MultivariateSpectralAnalysis {
    inner: CoreMultiSpec,
}

#[pymethods]
impl MultivariateSpectralAnalysis {
    #[new]
    #[pyo3(signature = (window = 64, embedding = 3, nfft = 64, update_every = 16))]
    fn new(window: usize, embedding: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(embedding, "embedding")?;
        Ok(Self { inner: CoreMultiSpec::new(window, embedding, nfft, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Parzen-Windowed Periodogram.
///
/// Periodogram with the Parzen (de la Vallée Poussin) cubic-spline window:
///
/// ```text
/// w(n) = 1 - 6 (|n|/N)^2 + 6 (|n|/N)^3   for |n| <= N/2
///      = 2 (1 - |n|/N)^3                  for N/2 < |n| <= N
/// ```
#[pyclass(module = "qstream")]
pub struct ParzenPeriodogram {
    inner: CoreParzen,
}

#[pymethods]
impl ParzenPeriodogram {
    #[new]
    #[pyo3(signature = (window = 64, update_every = 16))]
    fn new(window: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(update_every, "update_every")?;
        Ok(Self { inner: CoreParzen::new(window, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Partial Coherence.
///
/// Coherence after removing each signal's dependence on its own one-lag past:
///
/// ```text
/// r_x = x_t - b_x x_{t-1},  r_y = y_t - b_y y_{t-1}
/// |gamma_xy|^2 = |S_{r_x r_y}|^2 / (S_{r_x r_x} S_{r_y r_y})
/// ```
#[pyclass(module = "qstream")]
pub struct PartialCoherence {
    inner: CorePartialCoh,
}

#[pymethods]
impl PartialCoherence {
    #[new]
    #[pyo3(signature = (window = 64, update_every = 16))]
    fn new(window: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(update_every, "update_every")?;
        Ok(Self { inner: CorePartialCoh::new(window, update_every) })
    }
    fn update(&mut self, x: f64, y: f64) -> PyResult<Option<SpectrumResult>> {
        chk2(x, y)?;
        Ok(self.inner.update(x, y).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Spectral Envelope.
///
/// Envelope via cepstral smoothing of the log-magnitude spectrum:
///
/// ```text
/// env(f) = exp( IFFT( lowpass( FFT( log|X(f)| ) ) ) )
/// ```
#[pyclass(module = "qstream")]
pub struct SpectralEnvelope {
    inner: CoreSpecEnv,
}

#[pymethods]
impl SpectralEnvelope {
    #[new]
    #[pyo3(signature = (window = 64, cepstral_order = 8, update_every = 16))]
    fn new(window: usize, cepstral_order: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(cepstral_order, "cepstral_order")?;
        Ok(Self { inner: CoreSpecEnv::new(window, cepstral_order, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Wiener-Hopf Optimal Filter.
///
/// FIR smoother solving the Wiener-Hopf normal equations:
///
/// ```text
/// R h = p
/// R[i][j] = r_xx(|i - j|),  p[i] = r_xd(i)
/// y_t = sum_i h[i] x_{t-i}
/// ```
///
/// `update` returns the filtered value once warm; `.coefficients` exposes the
/// designed FIR taps.
#[pyclass(module = "qstream")]
pub struct WienerHopfFilter {
    inner: CoreWienerHopf,
}

#[pymethods]
impl WienerHopfFilter {
    #[new]
    #[pyo3(signature = (window = 64, order = 4, update_every = 8))]
    fn new(window: usize, order: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(order, "order")?;
        Ok(Self { inner: CoreWienerHopf::new(window, order, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<f64>> {
        chk(value)?;
        Ok(self.inner.update(value))
    }
    #[getter]
    fn coefficients(&self) -> Vec<f64> {
        self.inner.coefficients().to_vec()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Capon APES (Amplitude and Phase Estimation) Spectrum.
///
/// Narrowband filter-bank spectrum with unit-gain constraint:
///
/// ```text
/// alpha(omega) = (a^H Q^{-1} g) / (a^H Q^{-1} a)
/// P(omega) = |alpha(omega)|^2
/// ```
#[pyclass(module = "qstream")]
pub struct ApesSpectrum {
    inner: CoreApes,
}

#[pymethods]
impl ApesSpectrum {
    #[new]
    #[pyo3(signature = (window = 64, order = 4, nfft = 128, update_every = 16))]
    fn new(window: usize, order: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(order, "order")?;
        Ok(Self { inner: CoreApes::new(window, order, nfft, update_every) })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}
