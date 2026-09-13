# Workbook map

The <a href="/low_latency_finance_signal_api.xlsx" download>source workbook</a> is qstream's planning catalog. Its function names are descriptive research names, not necessarily Python export names. Use the [API catalog](/api/) for exact callable names, signatures, and current behavior.

The workbook's *Wickra Status* column records an earlier comparison with Wickra; it is not a current qstream implementation-status field. The table below includes the workbook's suggested module, streaming model, and priority without treating that old column as current status.

## Finance

| Function in workbook | Suggested module | Streaming model | Priority |
|---|---|---|---|
| Abdi-Ranaldo Closing-Price Spread Estimator | Microstructure | Rolling/window | High |
| Corwin-Schultz High-Low Bid-Ask Spread Estimator | Microstructure | Rolling/window | High |
| Glosten-Milgrom Bid-Ask Spread Model | Microstructure | Recursive/state | Medium |
| Garman-Klass Volatility | Volatility | Rolling/window | High |
| Parkinson Volatility Estimator | Volatility | Rolling/window | High |
| Rogers-Satchell Drift-Independent Volatility | Volatility | Rolling/window | High |
| RiskMetrics EWMA Volatility Forecast | Volatility | O(1) recursive | High |
| GARCH(1,1) Conditional Volatility Forecast | Volatility | O(1) recursive | High |
| Realized Volatility | Volatility | Accumulator/window | High |
| Realized Kernel with Microstructure Noise Correction | Volatility | Bounded lags/window | Medium |
| Two-Scale Realized Variance | Volatility | Window/subsamples | Medium |
| Rolling Volatility | Volatility | Rolling moments | High |
| Rolling Returns | Returns | Ring buffer | High |
| Value-at-Risk (VaR) | Tail Risk | Rolling distribution | High |
| Conditional Value at Risk | Tail Risk | Rolling tail | High |
| Expected Shortfall (CVaR) | Tail Risk | Rolling tail | High |
| Cornish-Fisher Modified VaR (PerformanceAnalytics) | Tail Risk | Rolling moments | High |
| Value-at-Risk via Cornish-Fisher Expansion | Tail Risk | Rolling moments | High |
| Entropic Value-at-Risk (EVaR) | Tail Risk | Online moments + solve | Medium |
| Conditional Drawdown-at-Risk (CDaR) | Tail Risk | Rolling drawdowns | High |
| Rachev Ratio | Tail Risk | Rolling quantiles/tails | High |
| EVT Tail Risk: POT/GPD VaR and Expected Shortfall | Tail Risk | Rolling tail model | Medium |
| Johnson SU Value-at-Risk | Tail Risk | Rolling fit | Low |
| Spectral Risk Measure (Exponential) | Tail Risk | Rolling ordered losses | Medium |
| Component & Marginal VaR | Portfolio Risk | Covariance/state | High |
| EWMA Multivariate Covariance Matrix | Portfolio Risk | O(N²) recursive | High |
| Portfolio Returns Analysis | Portfolio Risk | O(N) | High |
| Portfolio Duration Analysis | Portfolio Risk | O(N) | Medium |
| Maximum Diversification Portfolio | Portfolio Analytics | Periodic optimization | Medium |
| Risk Parity (Equal Risk Contribution) | Portfolio Analytics | Periodic optimization | Medium |
| Exponentially Weighted Portfolio | Portfolio Analytics | Recursive | Medium |
| Factor Investing (Exposures & Contributions) | Factor Models | Online regression | High |
| Beta & Alpha (CAPM) | Factor Models | Online covariance | High |
| CAPM (Capital Asset Pricing Model) | Factor Models | Online regression | High |
| Jensen’s Alpha | Factor Models | Online regression | High |
| Rolling-Window Beta Stability | Factor Models | Rolling regression | High |
| Fama-French Three-Factor Model | Factor Models | Online regression | High |
| Fama-French Five-Factor Model | Factor Models | Online regression | High |
| Carhart Four-Factor Model | Factor Models | Online regression | High |
| Multi-Factor Models | Factor Models | Online regression | High |
| Treynor-Mazuy Market Timing Model | Factor Models | Online regression | Medium |
| Tracking Error | Performance | Rolling variance | High |
| Up/Down Capture Ratio | Performance | Conditional accumulators | High |
| Gain-Loss Ratio (Bernardo-Ledoit) | Performance | Conditional accumulators | High |
| Deflated Sharpe Ratio (Bailey-Lopez de Prado) | Performance | Rolling moments/state | High |
| Lo Autocorrelation-Adjusted Sharpe Ratio | Performance | Rolling autocovariance | High |
| Conditional Sharpe and Omega Ratio | Performance | Rolling distribution | High |
| Sortino Ratio with Downside Deviation | Performance | Rolling downside moments | High |
| Sortino Ratio with Minimum Acceptable Return (MAR) | Performance | Rolling downside moments | High |
| Fleming-Ostdiek-Whaley VIX Implied Volatility | Derivatives / Volatility | Snapshot/stream state | Medium |
| Vandermeer VIX-Implied Volatility Calculator | Derivatives / Volatility | Snapshot/stream state | Low |
| VIX Futures Term Structure | Derivatives / Volatility | Snapshot per update | High |
| Implied Volatility (Newton-Raphson) | Derivatives / Volatility | Per-tick solve | High |
| Variance Swap | Derivatives / Volatility | Incremental where possible | Medium |
| Demeterfi Variance Swap Replication | Derivatives / Volatility | Snapshot/window | Medium |

## Signal processing

| Function in workbook | Suggested module | Streaming model | Priority |
|---|---|---|---|
| Adaptive Kalman Filter (Online Q-R Tuning) | Kalman / State Space | Recursive | High |
| Adaptive Notch Filter Frequency Estimator | Adaptive Cycle | Recursive | High |
| Alpha-Beta (g-h) Tracker | Kalman / State Space | Recursive | High |
| ARMA Spectral Estimation (Burg / Yule-Walker Hybrid) | Spectral | Window | Medium |
| Autoregressive Spectral Density | Spectral | Window/state | High |
| Bicoherence | Higher-order Spectral | Window | Low |
| Bispectrum Analysis | Higher-order Spectral | Window | Low |
| Blackman-Tukey Spectral Estimation | Spectral | Window | Medium |
| Bootstrap Particle Filter | State Space | Particle state | Medium |
| Capon APES Amplitude and Phase Estimation | Spectral | Window | Low |
| Capon MVDR Spectral Estimator | Spectral | Window | Medium |
| Cepstral Analysis | Spectral | Window | Low |
| Chirp-Z Transform (Zoom FFT) | Spectral | Window | Medium |
| Cohen-Class Time-Frequency Distributions (Wigner-Ville) | Time-Frequency | Window | Low |
| Coherence Function | Cross-Asset Spectral | Window | High |
| Constant-Q Transform | Time-Frequency | Window | Medium |
| Continuous Wavelet Scalogram | Wavelet | Window | Medium |
| Continuous Wavelet Transform (Morlet) | Wavelet | Window | High |
| Cross-Spectral Analysis | Cross-Asset Spectral | Window | High |
| Cross-Spectral Density and Coherence | Cross-Asset Spectral | Window | High |
| Cross-Wavelet Transform | Cross-Asset Wavelet | Window | Medium |
| Cubature Kalman Filter | Kalman / State Space | Recursive | Medium |
| Daniell-Smoothed Periodogram | Spectral | Window | Medium |
| Discrete Wavelet Transform | Wavelet | Window/block | High |
| Dual-Tree Complex Wavelet Transform | Wavelet | Window/block | Medium |
| Eigenvector Frequency Estimator | Spectral | Window | Low |
| EKF Numerical-Derivative Jacobian Variant | Kalman / State Space | Recursive | Medium |
| Empirical Wavelet Transform (Gilles) | Wavelet | Window | Medium |
| Ensemble Kalman Filter | Kalman / State Space | Ensemble state | Medium |
| ESPRIT Frequency Estimation | Spectral | Window | Medium |
| Extended Kalman Filter | Kalman / State Space | Recursive | High |
| FastICA Blind Source Separation | Multivariate Signal | Window/online variants | Medium |
| FFT Spectral Density Estimation | Spectral | Window FFT | High |
| Fourier Transform (DFT & Power Spectrum) | Spectral | Window | High |
| Fractional Fourier Transform (FrFT) | Time-Frequency | Window | Low |
| Functional Correlation and Cross-Spectrum | Cross-Asset Spectral | Window | Medium |
| Goertzel Algorithm DFT | Spectral | Recursive per frequency | High |
| Goertzel Single-Tone Detection | Adaptive Cycle | Recursive | Medium |
| Higher-Order Spectra | Higher-order Spectral | Window | Low |
| Hilbert Transform Envelope Detection | Hilbert / Cycle | Window/filter | Medium |
| Hilbert Transform: Analytic Signal, Envelope and Instantaneous Phase | Hilbert / Cycle | Window/filter | Medium |
| Hilbert Transform: Instantaneous Phase and Frequency | Hilbert / Cycle | Window/filter | Medium |
| Hilbert-Huang Transform | EMD / Time-Frequency | Window | Medium |
| Instantaneous Frequency | Hilbert / Cycle | Recursive/window | High |
| Kolmogorov-Zurbenko Filter | Filtering | Cascaded moving windows | Medium |
| Kurtogram Spectral Kurtosis | Spectral Features | Window | Medium |
| Lattice Prediction-Error Filter | Prediction | Recursive | High |
| Levinson-Durbin Recursion | Prediction | Window/state | High |
| Linear Predictive Coding (LPC) Analysis | Prediction | Window/state | Medium |
| LMS Adaptive Filter | Adaptive Filter | Recursive | High |
| Local Mean Decomposition (LMD) | Decomposition | Window | Low |
| Matching Pursuit Greedy Sparse Decomposition | Sparse Decomposition | Window | Low |
| Matrix Pencil (Hua-Sarkar) Frequency Estimation | Spectral | Window | Medium |
| Maximal Overlap DWT | Wavelet | Window/block | High |
| Minimum-Norm Frequency Estimation | Spectral | Window | Low |
| Modified Covariance (Forward-Backward) AR Spectrum | Spectral | Window | Medium |
| Morlet Wavelet Analysis | Wavelet | Window | High |
| Multiple Coherence | Cross-Asset Spectral | Window | Medium |
| Multiresolution Analysis (Wavelet MRA) | Wavelet | Window/block | High |
| Multivariate Spectral Analysis | Cross-Asset Spectral | Window | High |
| MUSIC Spectral Estimation (High-Resolution Peaks) | Spectral | Window | Medium |
| Page-Hinkley Change Detector | Regime Detection | O(1) recursive | High |
| Partial Coherence | Cross-Asset Spectral | Window | Medium |
| Parzen Window Spectral Estimation | Spectral | Window | Low |
| Periodogram (Bartlett's Method) | Spectral | Window | High |
| Periodogram Frequency Analysis | Spectral | Window | High |
| Phase-Locked Loop Analysis | Adaptive Cycle | Recursive | Medium |
| Pisarenko Harmonic Decomposition | Spectral | Window | Low |
| Prony Harmonic Spectral Decomposition | Spectral | Window | Medium |
| Reassigned Spectrogram | Time-Frequency | Window | Low |
| RLS Adaptive Filter (Recursive Least Squares) | Adaptive Filter | Recursive | High |
| Savitzky-Golay Polynomial Smoothing | Filtering | Rolling window | High |
| Short-Time Fourier Transform (STFT) | Time-Frequency | Sliding FFT | High |
| Spectral Analysis | Spectral | Window | High |
| Spectral Coherence (mscohere / Welch) | Cross-Asset Spectral | Window | High |
| Spectral Density Estimation (Multitaper) | Spectral | Window | High |
| Spectral Envelope | Spectral Features | Window | Medium |
| Spectral Shape Features | Spectral Features | Window | High |
| Spectrogram Time-Frequency Analysis | Time-Frequency | Sliding FFT | High |
| Square Root Kalman Filter | Kalman / State Space | Recursive | Medium |
| Stationary (Undecimated) Wavelet Denoising | Wavelet | Window/block | High |
| Stockwell S-Transform | Time-Frequency | Window | Medium |
| SureShrink Wavelet Thresholding | Wavelet | Window/block | Medium |
| Synchrosqueezing Transform | Time-Frequency / Wavelet | Window | Medium |
| Teager-Kaiser Energy Operator | Regime / Energy | O(1) local | High |
| Thomson Multitaper PSD | Spectral | Window | High |
| Thomson's Multitaper Spectral Estimation (spec.mtm) | Spectral | Window | High |
| Tunable-Q Wavelet Transform | Wavelet | Window | Medium |
| Unscented Kalman Filter | Kalman / State Space | Recursive | High |
| Variational Mode Decomposition | Decomposition | Window | Medium |
| Wavelet Analysis | Wavelet | Window/block | High |
| Wavelet Coherence | Cross-Asset Wavelet | Window | High |
| Wavelet Correlation | Cross-Asset Wavelet | Window | High |
| Wavelet Decomposition (Discrete) | Wavelet | Window/block | High |
| Wavelet Packet Decomposition | Wavelet | Window/block | Medium |
| Wavelet Phase Synchrony | Cross-Asset Wavelet | Window | Medium |
| Wavelet Regression | Wavelet | Window | Medium |
| Wavelet Variance | Wavelet | Window | High |
| Welch Overlap Segmented Spectral Estimator for Narrowband Signals | Spectral | Window | Medium |
| Welch Overlapped Periodogram | Spectral | Window | High |
| Welch Power Spectrum | Spectral | Window | High |
| Welch's Power Spectral Density | Spectral | Window | High |
| Wiener Filter Noise Reduction | Filtering | Window/adaptive | High |
| Wiener-Hopf Optimal Filter | Filtering | Window/state | Medium |
| Zero-Crossing Rate Analysis | Regime / Cycle | O(1)/window | High |
