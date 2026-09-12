//! PyO3 wrappers for Phase 7 experimental features.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::python::result_types::{SpectrumResult, WaveletResult};
use crate::python::{chk, chk2, chk_pos, chk_slice};
use crate::signal::cycle::{FastICA as CoreFastICA, PhaseLockedLoop as CorePLL};
use crate::signal::decomposition::{
    DecompositionResult, EmdDecomposition as CoreEmd, LmdDecomposition as CoreLmd,
    MatchingPursuitDecomposition as CoreMP, SynchrosqueezingTransform as CoreSynchSqueeze,
    VmdDecomposition as CoreVmd,
};
use crate::signal::heavy_kalman::{
    CubatureKalman as CoreCKF, EnsembleKalman as CoreEnKF, ParticleFilter as CorePF,
};
use crate::signal::higher_order::{
    BicoherenceAnalysis as CoreBicoherence, BicoherenceResult,
    BispectrumAnalysis as CoreBispectrum, BispectrumResult,
    CumulantResult, HigherOrderCumulants as CoreHOC,
};
use crate::signal::spectral_heavy::{
    CaponSpectrum as CoreCapon, EspritSpectrum as CoreEsprit,
    MatrixPencilSpectrum as CoreMatrixPencil, MinimumNormSpectrum as CoreMinNorm,
    MusicSpectrum as CoreMusic, PisarenkoSpectrum as CorePisarenko, PronySpectrum as CoreProny,
};
use crate::signal::timefreq_extras::{
    ChirpZTransform as CoreChirpZ, ConstantQTransform as CoreCQT,
    FractionalFourierTransform as CoreFrFT, KurtogramAnalysis as CoreKurtogram,
    ReassignedSpectrogram as CoreReassigned, StockwellTransform as CoreStockwell,
    WignerVilleDistribution as CoreWV,
};
use crate::signal::wavelet_extra::{
    CrossWaveletTransform as CoreXWT, DualTreeCwt as CoreDTCWT,
    EmpiricalWaveletTransform as CoreEWT, StationaryWaveletDenoise as CoreStatDen,
    SureShrinkDenoise as CoreSureShrink, TunableQWavelet as CoreTQWT,
    WaveletPhaseSynchrony as CoreWPS, WaveletRegression as CoreWReg,
};

// ===========================================================================
// Decomposition Result
// ===========================================================================

#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct PyDecompositionResult {
    #[pyo3(get)]
    pub coefficients: Vec<f64>,
    #[pyo3(get)]
    pub n_modes: usize,
    #[pyo3(get)]
    pub mode_frequencies: Vec<f64>,
}

impl From<DecompositionResult> for PyDecompositionResult {
    fn from(r: DecompositionResult) -> Self {
        Self {
            coefficients: r.coefficients,
            n_modes: r.n_modes,
            mode_frequencies: r.mode_frequencies,
        }
    }
}

#[pymethods]
impl PyDecompositionResult {
    fn __repr__(&self) -> String {
        format!(
            "DecompositionResult(n_modes={}, n_coeffs={})",
            self.n_modes,
            self.coefficients.len()
        )
    }
}

// ===========================================================================
// Higher-Order Result Types
// ===========================================================================

#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct PyBispectrumResult {
    #[pyo3(get)]
    pub values: Vec<f64>,
    #[pyo3(get)]
    pub frequencies: Vec<f64>,
    #[pyo3(get)]
    pub entropy: f64,
    #[pyo3(get)]
    pub total_coupling: f64,
}

impl From<BispectrumResult> for PyBispectrumResult {
    fn from(r: BispectrumResult) -> Self {
        Self {
            values: r.values,
            frequencies: r.frequencies,
            entropy: r.entropy,
            total_coupling: r.total_coupling,
        }
    }
}

#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct PyBicoherenceResult {
    #[pyo3(get)]
    pub values: Vec<f64>,
    #[pyo3(get)]
    pub frequencies: Vec<f64>,
    #[pyo3(get)]
    pub peak: f64,
}

impl From<BicoherenceResult> for PyBicoherenceResult {
    fn from(r: BicoherenceResult) -> Self {
        Self {
            values: r.values,
            frequencies: r.frequencies,
            peak: r.peak,
        }
    }
}

#[pyclass(module = "qstream")]
#[derive(Debug)]
pub struct PyCumulantResult {
    #[pyo3(get)]
    pub skewness: f64,
    #[pyo3(get)]
    pub kurtosis: f64,
    #[pyo3(get)]
    pub nonlinearity_index: f64,
}

impl From<CumulantResult> for PyCumulantResult {
    fn from(r: CumulantResult) -> Self {
        Self {
            skewness: r.skewness,
            kurtosis: r.kurtosis,
            nonlinearity_index: r.nonlinearity_index,
        }
    }
}

// ===========================================================================
// Spectral Heavy Methods
// ===========================================================================

macro_rules! make_spectral_heavy {
    ($name:ident, $core:ty, $doc:literal) => {
        #[doc = $doc]
        #[pyclass(module = "qstream")]
        pub struct $name {
            inner: $core,
        }

        #[pymethods]
        impl $name {
            #[new]
            #[pyo3(signature = (window, n_sources, nfft, update_every))]
            fn new(window: usize, n_sources: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
                chk_pos(window, "window")?;
                chk_pos(n_sources, "n_sources")?;
                let inner = <$core>::new(window, n_sources, nfft, update_every);
                Ok(Self { inner })
            }
            fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
                chk(value)?;
                Ok(self.inner.update(value).map(Into::into))
            }
            fn update_many(&mut self, values: Vec<f64>) -> PyResult<Vec<Option<SpectrumResult>>> {
                chk_slice(&values)?;
                let mut out = Vec::with_capacity(values.len());
                for v in values {
                    out.push(self.update(v)?);
                }
                Ok(out)
            }
            fn reset(&mut self) {
                self.inner.reset();
            }
            fn __repr__(&self) -> String {
                stringify!($name).to_string()
            }
        }
    };
}

make_spectral_heavy!(MusicSpectrum, CoreMusic, "MUSIC pseudospectrum.\n\n```text\nP(f) = 1 / sum_{i=n_sources+1}^{p} |v_i^H e(f)|^2\n```\n\nEigendecomposes the autocorrelation matrix into signal + noise subspaces and inverts the noise-subspace projection.");
make_spectral_heavy!(EspritSpectrum, CoreEsprit, "ESPRIT pseudospectrum.\n\n```text\nS2 = S1 Phi ;  f_i = -angle(eig(Phi)) / (2 pi)\n```\n\nExploits the rotational invariance between two overlapping signal subspaces.");
make_spectral_heavy!(MatrixPencilSpectrum, CoreMatrixPencil, "Matrix Pencil (Hua-Sarkar) spectrum.\n\n```text\nY2 - z Y1 = 0 ;  z_i = eig( Y1^+ Y2 )\n```\n\nGeneralized eigenvalue decomposition of two Hankel matrices.");

// Capon has different parameter order
/// Capon (MVDR) spectral estimator.
///
/// ```text
/// P(f) = 1 / ( e(f)^H R^{-1} e(f) )
/// ```
///
/// Minimum-variance distortionless-response spectrum from the inverse
/// autocorrelation matrix.
#[pyclass(module = "qstream")]
pub struct CaponSpectrum {
    inner: CoreCapon,
}

#[pymethods]
impl CaponSpectrum {
    #[new]
    #[pyo3(signature = (window, ar_order, nfft, update_every))]
    fn new(window: usize, ar_order: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(ar_order, "ar_order")?;
        Ok(Self {
            inner: CoreCapon::new(window, ar_order, nfft, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

make_spectral_heavy!(PisarenkoSpectrum, CorePisarenko, "Pisarenko harmonic decomposition.\n\n```text\nP(f) = 1 / |sum_i v_min[i] e^{-j 2 pi f i}|^2\n```\n\nUses the minimum-eigenvalue eigenvector as a prediction-error filter.");
make_spectral_heavy!(MinimumNormSpectrum, CoreMinNorm, "Minimum-Norm frequency estimator.\n\n```text\nP(f) = 1 / |sum_i f_min[i] e^{-j 2 pi f i}|^2\n```\n\nNoise-subspace combination constrained to unit first tap.");
make_spectral_heavy!(PronySpectrum, CoreProny, "Prony's method spectrum.\n\n```text\nx_t = sum_i A_i e^{j w_i t} ;  P(f) = 1 / |1 + sum a_i e^{-j w i}|^2\n```\n\nFits damped complex exponentials via a linear-prediction polynomial.");

// ===========================================================================
// Decomposition
// ===========================================================================

/// Variational Mode Decomposition (VMD).
///
/// ```text
/// min_{u_k, w_k} sum_k || d_t[(delta + j/pi t) * u_k] e^{-j w_k t} ||^2
/// s.t. sum_k u_k = f
/// ```
///
/// Non-recursive decomposition into K band-limited intrinsic modes via ADMM.
#[pyclass(module = "qstream")]
pub struct VmdDecomposition {
    inner: CoreVmd,
}

#[pymethods]
impl VmdDecomposition {
    #[new]
    #[pyo3(signature = (window = 64, n_modes = 3, alpha = 2000.0, update_every = 16))]
    fn new(window: usize, n_modes: usize, alpha: f64, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(n_modes, "n_modes")?;
        chk2(alpha, 0.0)?;
        Ok(Self {
            inner: CoreVmd::new(window, n_modes, alpha, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<PyDecompositionResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Empirical Mode Decomposition (EMD / Hilbert-Huang).
///
/// ```text
/// x_t = sum_k IMF_k(t) + r(t)
/// ```
///
/// Sifting loop extracts intrinsic mode functions (zero-mean, one extrema
/// count) adaptively.
#[pyclass(module = "qstream")]
pub struct EmdDecomposition {
    inner: CoreEmd,
}

#[pymethods]
impl EmdDecomposition {
    #[new]
    #[pyo3(signature = (window = 64, max_imfs = 4, update_every = 16))]
    fn new(window: usize, max_imfs: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(max_imfs, "max_imfs")?;
        Ok(Self {
            inner: CoreEmd::new(window, max_imfs, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<PyDecompositionResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Local Mean Decomposition (LMD).
///
/// ```text
/// x_t = sum_k PF_k(t) ;  PF_k = a_k(t) * s_k(t)
/// ```
///
/// Decomposes into product functions of a smoothed envelope `a_k` and a purely
/// frequency-modulated signal `s_k`.
#[pyclass(module = "qstream")]
pub struct LmdDecomposition {
    inner: CoreLmd,
}

#[pymethods]
impl LmdDecomposition {
    #[new]
    #[pyo3(signature = (window = 64, max_pfs = 4, update_every = 16))]
    fn new(window: usize, max_pfs: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(max_pfs, "max_pfs")?;
        Ok(Self {
            inner: CoreLmd::new(window, max_pfs, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<PyDecompositionResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Matching Pursuit greedy sparse decomposition.
///
/// ```text
/// x = sum_k <R_k, g_k> g_k + R_{K+1}
/// ```
///
/// Iteratively selects the Gabor atom `g_k` best correlated with the residual
/// `R_k`.
#[pyclass(module = "qstream")]
pub struct MatchingPursuitDecomposition {
    inner: CoreMP,
}

#[pymethods]
impl MatchingPursuitDecomposition {
    #[new]
    #[pyo3(signature = (window = 64, n_atoms = 5, update_every = 16))]
    fn new(window: usize, n_atoms: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(n_atoms, "n_atoms")?;
        Ok(Self {
            inner: CoreMP::new(window, n_atoms, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<PyDecompositionResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Synchrosqueezed STFT.
///
/// ```text
/// S(f) = sum_t delta(f - omega(t)) |X(t, omega(t))|^2
/// ```
///
/// Reassigns STFT energy to the instantaneous-frequency estimate, sharpening
/// the time-frequency representation.
#[pyclass(module = "qstream")]
pub struct SynchrosqueezingTransform {
    inner: CoreSynchSqueeze,
}

#[pymethods]
impl SynchrosqueezingTransform {
    #[new]
    #[pyo3(signature = (window = 64, nfft = 128, update_every = 16))]
    fn new(window: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(nfft, "nfft")?;
        Ok(Self {
            inner: CoreSynchSqueeze::new(window, nfft, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

// ===========================================================================
// Higher-Order Spectra
// ===========================================================================

/// Bispectrum analysis.
///
/// ```text
/// B(f1, f2) = E[ X(f1) X(f2) X*(f1 + f2) ]
/// ```
///
/// Third-order spectrum detecting quadratic phase coupling between frequency
/// components.
#[pyclass(module = "qstream")]
pub struct BispectrumAnalysis {
    inner: CoreBispectrum,
}

#[pymethods]
impl BispectrumAnalysis {
    #[new]
    #[pyo3(signature = (window = 64, nfft = 64, update_every = 16))]
    fn new(window: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreBispectrum::new(window, nfft, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<PyBispectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Bicoherence analysis.
///
/// ```text
/// b^2(f1, f2) = |B(f1, f2)|^2 / (E|X(f1)X(f2)|^2 * E|X(f1+f2)|^2)
/// ```
///
/// Normalized bispectrum in [0, 1]; 1 indicates perfect quadratic coupling.
#[pyclass(module = "qstream")]
pub struct BicoherenceAnalysis {
    inner: CoreBicoherence,
}

#[pymethods]
impl BicoherenceAnalysis {
    #[new]
    #[pyo3(signature = (window = 64, nfft = 64, update_every = 16))]
    fn new(window: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreBicoherence::new(window, nfft, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<PyBicoherenceResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Higher-order cumulants (skewness and excess kurtosis).
///
/// ```text
/// skewness = m3 / m2^{3/2}
/// kurtosis = m4 / m2^2 - 3
/// ```
///
/// Third and fourth standardized moments of the return/signal distribution.
#[pyclass(module = "qstream")]
pub struct HigherOrderCumulants {
    inner: CoreHOC,
}

#[pymethods]
impl HigherOrderCumulants {
    #[new]
    #[pyo3(signature = (window = 64, update_every = 16))]
    fn new(window: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreHOC::new(window, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<PyCumulantResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

// ===========================================================================
// Time-Frequency Extras
// ===========================================================================

/// Stockwell S-transform.
///
/// ```text
/// S(tau, f) = int x(t) (|f|/sqrt(2pi)) e^{-(tau-t)^2 f^2 / 2} e^{-j 2pi f t} dt
/// ```
///
/// Hybrid STFT/wavelet with a frequency-dependent Gaussian window.
#[pyclass(module = "qstream")]
pub struct StockwellTransform {
    inner: CoreStockwell,
}

#[pymethods]
impl StockwellTransform {
    #[new]
    #[pyo3(signature = (window = 64, nfft = 64, update_every = 16))]
    fn new(window: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreStockwell::new(window, nfft, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Reassigned spectrogram.
///
/// ```text
/// S(f') = sum_t delta(f' - omega(t,f)) |X(t,f)|^2
/// ```
///
/// Moves spectrogram energy to the local center of gravity, sharpening the
/// time-frequency representation.
#[pyclass(module = "qstream")]
pub struct ReassignedSpectrogram {
    inner: CoreReassigned,
}

#[pymethods]
impl ReassignedSpectrogram {
    #[new]
    #[pyo3(signature = (window = 64, nfft = 64, update_every = 16))]
    fn new(window: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreReassigned::new(window, nfft, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Constant-Q Transform (CQT).
///
/// ```text
/// X(f_k) = sum_n x[n] w(n, f_k) e^{-j 2 pi f_k n}
/// f_k = f_min * 2^{k/bins}
/// ```
///
/// Logarithmically spaced frequency bins with constant Q (center frequency /
/// bandwidth ratio).
#[pyclass(module = "qstream")]
pub struct ConstantQTransform {
    inner: CoreCQT,
}

#[pymethods]
impl ConstantQTransform {
    #[new]
    #[pyo3(signature = (window = 64, n_bins = 24, f_min = 0.01, f_max = 0.4, update_every = 16))]
    fn new(
        window: usize,
        n_bins: usize,
        f_min: f64,
        f_max: f64,
        update_every: usize,
    ) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreCQT::new(window, n_bins, f_min, f_max, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Fractional Fourier Transform (FrFT).
///
/// ```text
/// F_a(u) = sqrt(1 - j cot a) e^{j pi u^2 cot a}
///          * int x(t) e^{j pi t^2 cot a} e^{-j 2 pi u t csc a} dt
/// ```
///
/// Rotates the signal by angle `a` in the time-frequency plane; `a = pi/2`
/// recovers the standard Fourier transform.
#[pyclass(module = "qstream")]
pub struct FractionalFourierTransform {
    inner: CoreFrFT,
}

#[pymethods]
impl FractionalFourierTransform {
    #[new]
    #[pyo3(signature = (window = 64, angle = 1.5707963267948966, update_every = 16))]
    fn new(window: usize, angle: f64, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreFrFT::new(window, angle, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Wigner-Ville Distribution.
///
/// ```text
/// W(t, f) = int x(t + tau/2) x*(t - tau/2) e^{-j 2 pi f tau} dtau
/// ```
///
/// Quadratic time-frequency distribution with the highest resolution but
/// cross-term interference for multi-component signals.
#[pyclass(module = "qstream")]
pub struct WignerVilleDistribution {
    inner: CoreWV,
}

#[pymethods]
impl WignerVilleDistribution {
    #[new]
    #[pyo3(signature = (window = 64, nfft = 64, update_every = 16))]
    fn new(window: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreWV::new(window, nfft, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Chirp-Z Transform (zoom FFT).
///
/// ```text
/// X(z_k) = sum_n x[n] z_k^{-n},  z_k = A W^{-k}
/// ```
///
/// Evaluates the Z-transform on a spiral contour, allowing high-resolution
/// zoom over a narrow frequency band.
#[pyclass(module = "qstream")]
pub struct ChirpZTransform {
    inner: CoreChirpZ,
}

#[pymethods]
impl ChirpZTransform {
    #[new]
    #[pyo3(signature = (window = 64, f_start = 0.1, f_end = 0.3, n_points = 64, update_every = 16))]
    fn new(
        window: usize,
        f_start: f64,
        f_end: f64,
        n_points: usize,
        update_every: usize,
    ) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreChirpZ::new(window, f_start, f_end, n_points, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Kurtogram (spectral kurtosis).
///
/// ```text
/// K(f) = E|X(f)|^4 / (E|X(f)|^2)^2 - 3
/// ```
///
/// Frequency-resolved kurtosis; high values flag bands containing transients
/// or impulsive components.
#[pyclass(module = "qstream")]
pub struct KurtogramAnalysis {
    inner: CoreKurtogram,
}

#[pymethods]
impl KurtogramAnalysis {
    #[new]
    #[pyo3(signature = (window = 64, nfft = 64, update_every = 16))]
    fn new(window: usize, nfft: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreKurtogram::new(window, nfft, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<SpectrumResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

// ===========================================================================
// Wavelet Extras
// ===========================================================================

/// Dual-Tree Complex Wavelet Transform (DTCWT).
///
/// ```text
/// psi_c = psi_real + j psi_imag   (Hilbert pair)
/// ```
///
/// Two parallel wavelet trees form an approximately analytic wavelet,
/// providing near shift-invariance and directionality.
#[pyclass(module = "qstream")]
pub struct DualTreeCwt {
    inner: CoreDTCWT,
}

#[pymethods]
impl DualTreeCwt {
    #[new]
    #[pyo3(signature = (window = 64, levels = 3, update_every = 16))]
    fn new(window: usize, levels: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(levels, "levels")?;
        Ok(Self {
            inner: CoreDTCWT::new(window, levels, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<WaveletResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Empirical Wavelet Transform (Gilles 2013).
///
/// ```text
/// psi_k(f) = bandpass wavelet built from detected Fourier-spectrum boundaries
/// ```
///
/// Builds an adaptive filter bank by detecting boundaries between spectral
/// modes and applying the resulting empirical wavelets.
#[pyclass(module = "qstream")]
pub struct EmpiricalWaveletTransform {
    inner: CoreEWT,
}

#[pymethods]
impl EmpiricalWaveletTransform {
    #[new]
    #[pyo3(signature = (window = 64, n_modes = 3, update_every = 16))]
    fn new(window: usize, n_modes: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(n_modes, "n_modes")?;
        Ok(Self {
            inner: CoreEWT::new(window, n_modes, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<WaveletResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Tunable-Q Wavelet Transform (TQWT).
///
/// ```text
/// Q = center_frequency / bandwidth
/// alpha = 1 - (Q - 1)/(Q + 1);  beta = 2/(Q + 1)
/// ```
///
/// Iterative two-channel filter bank with a tunable Q-factor; high Q for
/// oscillatory signals, low Q for transients.
#[pyclass(module = "qstream")]
pub struct TunableQWavelet {
    inner: CoreTQWT,
}

#[pymethods]
impl TunableQWavelet {
    #[new]
    #[pyo3(signature = (window = 64, levels = 4, q_factor = 2.0, update_every = 16))]
    fn new(window: usize, levels: usize, q_factor: f64, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(levels, "levels")?;
        if q_factor < 1.0 {
            return Err(PyValueError::new_err("q_factor must be >= 1"));
        }
        Ok(Self {
            inner: CoreTQWT::new(window, levels, q_factor, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<WaveletResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// SureShrink wavelet denoising.
///
/// ```text
/// threshold = sigma * sqrt(2 log N)   (soft thresholding)
/// y = sign(w) max(|w| - threshold, 0)
/// ```
///
/// Stein's Unbiased Risk Estimate threshold applied to wavelet coefficients.
#[pyclass(module = "qstream")]
pub struct SureShrinkDenoise {
    inner: CoreSureShrink,
}

#[pymethods]
impl SureShrinkDenoise {
    #[new]
    #[pyo3(signature = (window = 64, levels = 3, update_every = 16))]
    fn new(window: usize, levels: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(levels, "levels")?;
        Ok(Self {
            inner: CoreSureShrink::new(window, levels, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<WaveletResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Stationary (Undecimated) Wavelet Denoising.
///
/// ```text
/// a_{j+1}[t] = (a_j[t] + a_j[t + 2^j]) / 2
/// d_{j+1}[t] = (a_j[t] - a_j[t + 2^j]) / 2   (a trous)
/// ```
///
/// Shift-invariant `a trous` wavelet transform with thresholding of detail
/// coefficients.
#[pyclass(module = "qstream")]
pub struct StationaryWaveletDenoise {
    inner: CoreStatDen,
}

#[pymethods]
impl StationaryWaveletDenoise {
    #[new]
    #[pyo3(signature = (window = 64, levels = 3, update_every = 16))]
    fn new(window: usize, levels: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(levels, "levels")?;
        Ok(Self {
            inner: CoreStatDen::new(window, levels, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<WaveletResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

// Cross-wavelet and phase synchrony use pair interface
/// Cross-Wavelet Transform (XWT).
///
/// ```text
/// W_xy(a) = W_x(a) * W_y*(a)
/// ```
///
/// Cross-wavelet power revealing time-frequency regions of common power
/// between two signals.
#[pyclass(module = "qstream")]
pub struct CrossWaveletTransform {
    inner: CoreXWT,
}

#[pymethods]
impl CrossWaveletTransform {
    #[new]
    #[pyo3(signature = (window = 64, levels = 3, update_every = 16))]
    fn new(window: usize, levels: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(levels, "levels")?;
        Ok(Self {
            inner: CoreXWT::new(window, levels, update_every),
        })
    }
    fn update(&mut self, x: f64, y: f64) -> PyResult<Option<WaveletResult>> {
        chk2(x, y)?;
        Ok(self.inner.update(x, y).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Wavelet Phase Synchrony.
///
/// ```text
/// PLV = |1/N sum_t e^{j (phi_x(t) - phi_y(t))}|
/// ```
///
/// Phase-locking value across wavelet scales; near 1 means consistent phase
/// difference (synchronization), near 0 means none.
#[pyclass(module = "qstream")]
pub struct WaveletPhaseSynchrony {
    inner: CoreWPS,
}

#[pymethods]
impl WaveletPhaseSynchrony {
    #[new]
    #[pyo3(signature = (window = 64, levels = 3, update_every = 16))]
    fn new(window: usize, levels: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(levels, "levels")?;
        Ok(Self {
            inner: CoreWPS::new(window, levels, update_every),
        })
    }
    fn update(&mut self, x: f64, y: f64) -> PyResult<Option<WaveletResult>> {
        chk2(x, y)?;
        Ok(self.inner.update(x, y).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Wavelet Regression.
///
/// ```text
/// y = sum_j W_j^T (threshold_j(W_j))
/// ```
///
/// Denoises a signal via level-dependent wavelet shrinkage and reconstructs
/// the regression (smoothed) estimate.
#[pyclass(module = "qstream")]
pub struct WaveletRegression {
    inner: CoreWReg,
}

#[pymethods]
impl WaveletRegression {
    #[new]
    #[pyo3(signature = (window = 64, levels = 3, update_every = 16))]
    fn new(window: usize, levels: usize, update_every: usize) -> PyResult<Self> {
        chk_pos(window, "window")?;
        chk_pos(levels, "levels")?;
        Ok(Self {
            inner: CoreWReg::new(window, levels, update_every),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<Option<WaveletResult>> {
        chk(value)?;
        Ok(self.inner.update(value).map(Into::into))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

// ===========================================================================
// Heavy Kalman
// ===========================================================================

use crate::python::result_types::StateEstimate;

/// Bootstrap Particle Filter (Sequential Importance Resampling).
///
/// ```text
/// x_t^i ~ p(x_t | x_{t-1}^i)
/// w_t^i ~ p(y_t | x_t^i)
/// x_hat = sum_i w_t^i x_t^i
/// ```
///
/// Monte Carlo state estimator for nonlinear/non-Gaussian systems, with
/// systematic resampling when the effective sample size is low.
#[pyclass(module = "qstream")]
pub struct ParticleFilter {
    inner: CorePF,
}

#[pymethods]
impl ParticleFilter {
    #[new]
    #[pyo3(signature = (n_particles = 200, process_noise = 0.01, measurement_noise = 0.1, state_min = -5.0, state_max = 5.0))]
    fn new(
        n_particles: usize,
        process_noise: f64,
        measurement_noise: f64,
        state_min: f64,
        state_max: f64,
    ) -> PyResult<Self> {
        chk_pos(n_particles, "n_particles")?;
        chk2(process_noise, measurement_noise)?;
        Ok(Self {
            inner: CorePF::new(n_particles, process_noise, measurement_noise, state_min, state_max),
        })
    }
    fn update(&mut self, measurement: f64) -> PyResult<StateEstimate> {
        chk(measurement)?;
        Ok(self.inner.update(measurement).into())
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Ensemble Kalman Filter (EnKF).
///
/// ```text
/// x_hat = (1/N) sum_i x^i
/// P = (1/(N-1)) sum_i (x^i - x_hat)(x^i - x_hat)^T
/// x^i <- x^i + K (y - H x^i)
/// ```
///
/// Monte Carlo Kalman filter replacing the covariance with the sample
/// covariance of an ensemble.
#[pyclass(module = "qstream")]
pub struct EnsembleKalman {
    inner: CoreEnKF,
}

#[pymethods]
impl EnsembleKalman {
    #[new]
    #[pyo3(signature = (n_members = 50, state_dim = 2, process_noise = 0.01, measurement_noise = 0.1))]
    fn new(
        n_members: usize,
        state_dim: usize,
        process_noise: f64,
        measurement_noise: f64,
    ) -> PyResult<Self> {
        chk_pos(n_members, "n_members")?;
        chk_pos(state_dim, "state_dim")?;
        Ok(Self {
            inner: CoreEnKF::new(n_members, state_dim, process_noise, measurement_noise),
        })
    }
    fn update(&mut self, measurement: f64) -> PyResult<StateEstimate> {
        chk(measurement)?;
        Ok(self.inner.update(measurement).into())
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Cubature Kalman Filter (CKF).
///
/// ```text
/// X_i = mean +/- sqrt(n P) e_i     (cubature points)
/// mean = (1/2n) sum_i f(X_i)
/// P = (1/2n) sum_i (f(X_i) - mean)(f(X_i) - mean)^T + Q
/// ```
///
/// Deterministic sampling (cubature rule) for nonlinear filtering; accurate
/// to third order.
#[pyclass(module = "qstream")]
pub struct CubatureKalman {
    inner: CoreCKF,
}

#[pymethods]
impl CubatureKalman {
    #[new]
    #[pyo3(signature = (state_dim = 2, process_noise = 0.01, measurement_noise = 0.1))]
    fn new(state_dim: usize, process_noise: f64, measurement_noise: f64) -> PyResult<Self> {
        chk_pos(state_dim, "state_dim")?;
        chk2(process_noise, measurement_noise)?;
        Ok(Self {
            inner: CoreCKF::new(state_dim, process_noise, measurement_noise),
        })
    }
    fn update(&mut self, measurement: f64) -> PyResult<StateEstimate> {
        chk(measurement)?;
        Ok(self.inner.update(measurement).into())
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

// ===========================================================================
// Cycle: PLL
// ===========================================================================

/// Phase-Locked Loop (PLL) frequency estimator.
///
/// ```text
/// e = -x * sin(phi)             (phase detector)
/// f <- f + kp e + ki int(e)     (PI loop filter)
/// phi <- phi + 2 pi f           (NCO)
/// ```
///
/// Feedback loop that locks onto the frequency and phase of a sinusoid.
#[pyclass(module = "qstream")]
pub struct PhaseLockedLoop {
    inner: CorePLL,
}

#[pymethods]
impl PhaseLockedLoop {
    #[new]
    #[pyo3(signature = (freq0 = 0.1, kp = 0.2, ki = 0.01))]
    fn new(freq0: f64, kp: f64, ki: f64) -> PyResult<Self> {
        chk2(kp, ki)?;
        Ok(Self {
            inner: CorePLL::new(freq0, kp, ki),
        })
    }
    fn update(&mut self, value: f64) -> PyResult<(f64, f64, f64)> {
        chk(value)?;
        let (freq, i, q) = self.inner.update(value);
        Ok((freq, i, q))
    }
    fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

// ===========================================================================
// FastICA
// ===========================================================================

/// FastICA Blind Source Separation.
///
/// ```text
/// w <- E{x g(w^T x)} - E{g'(w^T x)} w   (fixed-point iteration)
/// s = W x
/// ```
///
/// Recovers independent components by maximizing non-Gaussianity (negentropy
/// approximation).
#[pyclass(module = "qstream")]
pub struct FastICA {
    inner: CoreFastICA,
}

#[pymethods]
impl FastICA {
    #[new]
    #[pyo3(signature = (n_signals = 2, n_components = 2, window = 64, update_every = 16))]
    fn new(
        n_signals: usize,
        n_components: usize,
        window: usize,
        update_every: usize,
    ) -> PyResult<Self> {
        chk_pos(n_signals, "n_signals")?;
        chk_pos(n_components, "n_components")?;
        chk_pos(window, "window")?;
        Ok(Self {
            inner: CoreFastICA::new(n_signals, n_components, window, update_every),
        })
    }
    fn update(&mut self, signals: Vec<f64>) -> PyResult<Option<Vec<Vec<f64>>>> {
        chk_slice(&signals)?;
        Ok(self.inner.update(&signals))
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}
