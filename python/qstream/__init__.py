"""qstream: low-latency finance + signal-processing features.

Rust core (PyO3 + maturin); Python is the control/API layer. No NumPy, pandas,
SciPy, or Numba runtime dependency.

Streaming usage::

    from qstream import EwmaVolatility
    ind = EwmaVolatility(alpha=0.06)
    for r in returns:
        vol = ind.update(r)

Grouped execution (amortizes the Python<->Rust crossing)::

    from qstream import FeatureEngine
    engine = FeatureEngine(
        names=["vol", "garch", "ph"],
        kinds=["ewma_volatility", "garch", "page_hinkley"],
        channels=["return", "return", "close"],
    )
    features = engine.update(open, high, low, close, volume)
"""

from qstream._qstream import *  # noqa: F401,F403
from qstream import _qstream as _native

__version__ = _native.__version__

# API aliases / convenience names.
ExpectedShortfall = _native.ConditionalValueAtRisk  # CVaR == Expected Shortfall
CVaR = _native.ConditionalValueAtRisk
ES = _native.ConditionalValueAtRisk
WelchPSD = _native.WelchPsd
BetaAlpha = _native.Capm
FamaFrenchThree = _native.FamaFrench3
FamaFrenchFive = _native.FamaFrench5
Carhart = _native.Carhart4
UpDownCapture = _native.CaptureRatios
DWT = _native.DiscreteWaveletTransform
MODWT = _native.Modwt
CWT = _native.CwtMorlet
RLS = _native.RlsFilter
LMS = _native.LmsFilter

# Build __all__ from the public native names plus the aliases above.
__all__ = sorted(
    {n for n in dir(_native) if not n.startswith("_")}
    | {
        "ExpectedShortfall",
        "CVaR",
        "ES",
        "WelchPSD",
        "BetaAlpha",
        "FamaFrenchThree",
        "FamaFrenchFive",
        "Carhart",
        "UpDownCapture",
        "DWT",
        "MODWT",
        "CWT",
        "RLS",
        "LMS",
        "__version__",
    }
)
