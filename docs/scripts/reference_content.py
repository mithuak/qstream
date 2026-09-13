"""Human-authored explanations used by generated indicator reference pages."""

from __future__ import annotations

import re

USE_CASES = {
    "Finance · volatility": "Monitor changing volatility for risk limits, position sizing, or volatility forecasts.",
    "Finance · microstructure": "Estimate trading frictions, spread, or price impact from market observations.",
    "Finance · tail risk": "Track downside exposure and compare risk across return histories.",
    "Finance · factors": "Estimate an asset's sensitivity to benchmark or factor returns and attribute performance.",
    "Finance · performance": "Evaluate return quality relative to risk, downside, or a benchmark.",
    "Finance · portfolio": "Combine asset observations into portfolio-level returns or risk measures.",
    "Finance · derivatives": "Monitor option or volatility-linked instrument prices and exposures.",
    "Finance · portfolio optimizers": "Derive portfolio weights using a specified risk or diversification objective.",
    "Finance · realized volatility": "Estimate variation in high-frequency returns while accounting for sampling effects.",
    "Finance · volatility derivatives": "Analyze volatility index or variance-swap quantities.",
    "Signal · filters": "Smooth or adapt a noisy series before measurement or prediction.",
    "Signal · kalman / state-space / adaptive": "Track hidden state or predict the next observation from noisy measurements.",
    "Signal · regime / energy": "Detect changes in the behavior or energy of a stream.",
    "Signal · spectral": "Find periodic components or track how signal power is distributed by frequency.",
    "Signal · Spectral and filtering": "Inspect frequency content or construct a filter from a rolling signal window.",
    "Signal · time-frequency (STFT / Hilbert)": "Track how oscillation, envelope, or frequency changes over time.",
    "Signal · wavelets": "Inspect signal behavior at more than one time scale.",
    "Signal · prediction": "Model short-horizon dynamics or forecast a future sample.",
    "Spectral heavy methods": "Resolve closely spaced spectral components when a simpler periodogram is insufficient.",
    "Decomposition": "Separate a composite signal into simpler oscillatory or sparse components.",
    "Higher-order spectra": "Inspect nonlinear or non-Gaussian interactions among frequencies.",
    "Time-frequency": "Locate changing frequency content in time.",
    "Wavelet extras": "Analyze or denoise structure across time scales.",
    "Heavy Kalman": "Estimate latent state when a simpler linear Gaussian filter is not suitable.",
    "Cycle": "Track a changing phase or oscillation in a stream.",
    "FastICA": "Separate mixed signals into approximately independent components.",
    "Grouped execution": "Update several scalar features from one OHLCV bar with one Python-to-Rust call.",
}

CONSTRUCTOR_MEANINGS = {
    "period": "Rolling length or effective horizon; see the formula for this indicator's convention.",
    "window": "Number of recent samples retained for each calculation.",
    "update_every": "Number of updates between full recalculations after the window is ready.",
    "fs": "Sampling frequency in samples per unit time.",
    "window_type": "Taper applied to the signal window.",
    "alpha": "Smoothing or model coefficient; its exact role is defined in the formula above.",
    "beta": "Model coefficient; its exact role is defined in the formula above.",
    "lambda": "Exponential decay or forgetting parameter.",
    "lam": "Forgetting parameter for recursive least squares.",
    "omega": "Baseline term in the volatility recurrence.",
    "p": "Tail probability or probability threshold; see the indicator formula.",
    "confidence": "Confidence level for the reported risk estimate.",
    "nfft": "FFT length or spectral grid size.",
    "order": "Model, filter, or polynomial order.",
    "levels": "Number of wavelet decomposition scales.",
    "n_assets": "Number of assets expected in each input vector.",
    "n_factors": "Number of explanatory factor streams.",
    "n_sources": "Number of spectral sources to estimate.",
    "n_signals": "Number of simultaneous input signals.",
    "n_components": "Number of independent components to return.",
    "risk_free": "Risk-free return used as the performance baseline.",
    "annualization": "Scale factor used to annualize the result.",
    "threshold": "Decision threshold for the detector or filter.",
    "scales": "Wavelet scales to evaluate.",
    "names": "Labels for the engine's outputs, in order.",
    "kinds": "Registered scalar feature kinds, in the same order as names.",
    "channels": "OHLCV or derived-return source for each feature.",
    "params": "Optional numeric parameter for each configured feature.",
    "adapt": "Rate at which the filter adjusts its noise estimates.",
    "angle": "Rotation angle of the fractional Fourier transform.",
    "ar_order": "Order of the autoregressive model used in the spectrum estimate.",
    "beta_window": "Rolling window used to estimate each beta.",
    "block": "Number of samples in one Goertzel analysis block.",
    "cepstral_order": "Number of cepstral terms retained in the envelope.",
    "delta": "Allowed mean drift before the change score accumulates.",
    "deriv": "Order of the derivative to estimate; zero requests smoothing.",
    "dt": "Time interval between observations.",
    "embedding": "Number of delayed coordinates in the multivariate embedding.",
    "f_start": "Lower endpoint of the zoom-frequency range.",
    "f_end": "Upper endpoint of the zoom-frequency range.",
    "f_min": "Lowest analyzed frequency.",
    "f_max": "Highest analyzed frequency.",
    "freq0": "Initial or target normalized frequency for the adaptive tracker.",
    "frequencies": "Frequencies at which the detector evaluates power.",
    "gamma": "Risk-aversion weight in the spectral risk measure.",
    "hop": "Number of samples between adjacent transform frames.",
    "ki": "Integral gain for phase/frequency correction.",
    "kp": "Proportional gain for phase correction.",
    "m": "Smoothing span for the Daniell periodogram.",
    "mar": "Minimum acceptable return for the downside comparison.",
    "max_frames": "Maximum spectrogram frames retained.",
    "max_imfs": "Maximum intrinsic mode functions to extract.",
    "max_lag": "Largest autocovariance lag included in the estimate.",
    "max_pfs": "Maximum product functions to extract.",
    "measurement_noise": "Measurement-noise variance or scale.",
    "mu": "Adaptation step size for the filter.",
    "n_atoms": "Maximum number of atoms selected by matching pursuit.",
    "n_bins": "Number of frequency bins to analyze.",
    "n_lags": "Number of autocovariance lags used by the realized kernel.",
    "n_members": "Number of members in the state ensemble.",
    "n_modes": "Number of modes requested from the decomposition.",
    "n_particles": "Number of particles used to approximate the state distribution.",
    "n_points": "Number of output points in the zoom transform.",
    "n_subsamples": "Number of interleaved subsamples in the two-scale estimate.",
    "n_trials": "Number of selection trials accounted for in deflation.",
    "noise_window": "Window used to estimate local noise level.",
    "nonlinear": "Enable the built-in nonlinear state/observation model.",
    "nw": "Time-bandwidth product for multitaper analysis.",
    "p0": "Initial state covariance.",
    "passes": "Number of repeated smoothing passes.",
    "process_noise": "Process-noise variance or scale.",
    "q": "Process-noise variance.",
    "q_factor": "Quality factor controlling the wavelet bandwidth.",
    "r": "Measurement-noise variance.",
    "rho": "Pole radius controlling the notch bandwidth.",
    "s0": "Initial covariance factor.",
    "stability_window": "Window over which beta stability is summarized.",
    "state_dim": "Number of latent state dimensions.",
    "state_min": "Lower bound of the particle state range.",
    "state_max": "Upper bound of the particle state range.",
    "tapers": "Number of orthogonal tapers to average.",
    "threshold_quantile": "Quantile above which observations enter the extreme-value tail fit.",
}

INPUT_MEANINGS = {
    "open": "Opening price for the current bar.",
    "high": "Highest price in the current bar.",
    "low": "Lowest price in the current bar.",
    "close": "Closing price for the current bar.",
    "volume": "Volume for the current bar.",
    "asset_return": "Return of the asset being analyzed.",
    "market_return": "Return of the market benchmark.",
    "benchmark_return": "Return of the comparison benchmark.",
    "market": "Market factor return.",
    "smb": "Size factor return (small minus big).",
    "hml": "Value factor return (high minus low).",
    "rmw": "Profitability factor return (robust minus weak).",
    "cma": "Investment factor return (conservative minus aggressive).",
    "mom": "Momentum factor return.",
    "risk_free": "Risk-free return for this observation.",
    "measurement": "New noisy observation of the tracked state.",
    "returns": "Return vector for the current observation.",
    "weights": "Portfolio weights aligned with the asset vector.",
    "durations": "Duration of each asset in the portfolio.",
    "factors": "Factor-return vector for this observation.",
    "signals": "Vector of simultaneous signal observations.",
    "x": "Current observation from the first series.",
    "y": "Current observation from the second series.",
    "trade_price": "Observed execution price.",
    "bid": "Current bid quote.",
    "ask": "Current ask quote.",
    "spot": "Underlying spot price.",
    "strike": "Option strike price.",
    "ttm": "Time to maturity in years.",
    "rate": "Continuously compounded risk-free rate.",
    "market_price": "Observed market price of the option.",
    "sigma": "Volatility assumed by the option-pricing model.",
    "is_call": "True for a call, false for a put.",
    "spot_vix": "Current spot VIX level.",
    "futures": "Vector of VIX futures prices.",
    "maturities": "Time to maturity for each futures contract.",
    "spectrum": "SpectrumResult to summarize into shape features.",
    "acf": "Autocorrelation sequence for the recursion.",
    "covariance": "Covariance matrix used to calculate portfolio weights.",
    "data": "Observed signal samples used to fit the AR model.",
    "order": "Requested autoregressive model order.",
    "n_assets": "Number of assets represented by the covariance matrix.",
    "iters": "Maximum iterations of the risk-parity weight solver.",
}

OUTPUT_MEANINGS = {
    "FactorResult": "Regression coefficients and fit statistics.",
    "SpectrumResult": "Frequency bins, power values, dominant frequency, and peak power.",
    "WaveletResult": "Wavelet coefficients and scale information.",
    "StateEstimate": "Estimated state, velocity, and covariance.",
    "ChangeResult": "Whether a change was detected and its score.",
    "FrequencyEstimate": "Estimated frequency and filtered sample.",
    "PredictionResult": "Prediction and fitted coefficients.",
    "CaptureResult": "Up-market and down-market capture ratios.",
    "HilbertResult": "Instantaneous amplitude, phase, and frequency.",
}

FIELD_MEANINGS = {
    "frequencies": "Frequency coordinates for the spectral values.",
    "power": "Estimated power at each frequency.",
    "dominant_frequency": "Frequency of the strongest estimated component.",
    "peak_power": "Power of the strongest estimated component.",
    "alpha": "Estimated intercept or excess return after factor adjustment.",
    "betas": "Estimated factor loadings, in input factor order.",
    "r2": "Fraction of variation explained by the model.",
    "residual_variance": "Estimated unexplained variance.",
    "value": "Estimated scalar value.",
    "velocity": "Estimated rate of change, when available.",
    "covariance": "State uncertainty or covariance values.",
    "changed": "Whether the detector signaled a change.",
    "score": "Current change-detection score.",
    "prediction": "One-step prediction.",
    "coefficients": "Model or transform coefficients.",
    "amplitude": "Instantaneous signal envelope.",
    "phase": "Instantaneous phase.",
    "frequency": "Instantaneous frequency.",
    "up_capture": "Return capture in up-market observations.",
    "down_capture": "Return capture in down-market observations.",
    "scales": "Wavelet scales associated with the coefficients.",
    "slope": "Term-structure slope.",
    "curvature": "Term-structure curvature.",
    "contango": "Nearest futures price relative to spot VIX, as a fractional premium or discount.",
    "centroid": "Power-weighted center frequency.",
    "bandwidth": "Spread of spectral power around the centroid.",
    "entropy": "Entropy or concentration measure of the output distribution.",
    "rolloff": "Frequency below which a specified fraction of total power lies.",
    "flatness": "Ratio of geometric to arithmetic mean spectral power.",
    "total_var": "Portfolio-level value at risk.",
    "marginal": "Marginal risk contribution for each asset.",
    "component": "Weighted risk contribution for each asset.",
    "filtered": "Signal after adaptive filtering.",
    "n_modes": "Number of extracted signal modes.",
    "mode_frequencies": "Estimated frequency of each extracted mode.",
    "values": "Computed spectral or higher-order statistic values.",
    "total_coupling": "Aggregate strength of estimated frequency coupling.",
    "peak": "Largest estimated bicoherence value.",
    "skewness": "Third-order standardized asymmetry measure.",
    "kurtosis": "Fourth-order tail or peakedness measure.",
    "nonlinearity_index": "Summary measure of nonlinearity from higher-order cumulants.",
    "strike": "Calculated variance-swap strike.",
    "realized_variance": "Variance realized over the observed window.",
    "correction": "Adjustment term in the variance-swap calculation.",
    "var": "Estimated loss threshold at the requested tail probability.",
    "expected_shortfall": "Expected loss beyond the VaR threshold.",
    "shape": "Shape parameter of the fitted extreme-value tail.",
    "scale": "Scale parameter of the fitted extreme-value tail.",
    "n_exceedances": "Number of observations used in the tail fit.",
}

EXAMPLES = {
    "EwmaVolatility": """from qstream import EwmaVolatility

vol = EwmaVolatility(alpha=0.06)
for daily_return in [0.01, -0.005, 0.012]:
    print(vol.update(daily_return))""",
    "RealizedVolatility": """from qstream import RealizedVolatility

rv = RealizedVolatility(period=3)
for daily_return in [0.01, -0.005, 0.012, 0.004]:
    value = rv.update(daily_return)
    if value is not None:
        print(value)""",
    "GarmanKlass": """from qstream import GarmanKlass

gk = GarmanKlass(period=2)
for bar in [(100.0, 102.0, 99.0, 101.0), (101.0, 103.0, 100.0, 102.0)]:
    print(gk.update(*bar))  # open, high, low, close""",
    "RollingBeta": """from qstream import RollingBeta

beta = RollingBeta(period=3)
for asset_return, market_return in [(0.01, 0.008), (-0.005, -0.004), (0.012, 0.009)]:
    print(beta.update(asset_return, market_return))""",
    "ValueAtRisk": """from qstream import ValueAtRisk

var = ValueAtRisk(period=3, p=0.05)
for daily_return in [0.01, -0.02, 0.005, -0.01]:
    print(var.update(daily_return))""",
    "FamaFrench3": """from qstream import FamaFrench3

model = FamaFrench3(period=252)
result = model.update(0.01, 0.008, 0.002, -0.001, risk_free=0.0)
print(result.alpha, result.betas, result.r2)""",
    "WelchPsd": """from qstream import WelchPsd

psd = WelchPsd(window=64, update_every=8)
for sample in [float(i % 10) for i in range(96)]:
    result = psd.update(sample)
    if result is not None:
        print(result.dominant_frequency, result.peak_power)""",
    "PageHinkley": """from qstream import PageHinkley

detector = PageHinkley(delta=0.005, threshold=1.0)
for value in [0.0, 0.1, 0.0, 2.0]:
    result = detector.update(value)
    print(result.changed, result.score)""",
}

PARAMETER_OVERRIDES = {
    ("EwmaVolatility", "alpha"): "Weight on the newest squared return (`lambda = 1 - alpha`).",
    ("EwmaVariance", "alpha"): "Weight on the previous variance in the recurrence.",
    ("Garch", "omega"): "Baseline conditional-variance term.",
    ("Garch", "alpha"): "Weight on the previous squared return shock.",
    ("Garch", "beta"): "Weight on the previous conditional variance.",
    ("AlphaBetaTracker", "alpha"): "Gain applied to the measurement residual in the position estimate.",
    ("AlphaBetaTracker", "beta"): "Gain applied to the measurement residual in the velocity estimate.",
    ("ConditionalDrawdownAtRisk", "alpha"): "Confidence level for the drawdown tail estimate.",
    ("EntropicVaR", "alpha"): "Tail probability used in the entropic VaR bound.",
    ("EVTGpdTailRisk", "alpha"): "Tail level used in the GPD risk estimate.",
    ("JohnsonSUVaR", "alpha"): "Quantile level used for the Johnson-SU VaR estimate.",
    ("VmdDecomposition", "alpha"): "Bandwidth penalty in the VMD optimization.",
}


def why_use(group: str) -> str:
    group = group.removeprefix("Experimental · ")
    return USE_CASES.get(group, "Apply this calculation to the corresponding stream or portfolio analysis.")


def computation(doc: str | None) -> str:
    """Take the explanatory prose after the formula block, if available."""
    if not doc:
        return "See the formula and input/output contract below."
    clean = re.sub(r"```[\s\S]*?```", "", doc).strip()
    paragraphs = [" ".join(p.split()) for p in clean.split("\n\n") if p.strip()]
    return paragraphs[-1] if len(paragraphs) > 1 else paragraphs[0]


def input_meaning(name: str, group: str) -> str:
    if name == "value":
        if group.startswith("Finance ·"):
            return "Next return or scalar financial observation; check the formula for the required unit."
        return "Next sample of the signal or time series."
    return INPUT_MEANINGS.get(name, "Value for this update; see the formula and signature for its role.")


def parameter_meaning(indicator: str, parameter: str) -> str:
    return PARAMETER_OVERRIDES.get(
        (indicator, parameter),
        CONSTRUCTOR_MEANINGS.get(parameter, "See the formula and constructor signature for this setting."),
    )
