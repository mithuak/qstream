//! Signal-processing feature modules (pure Rust, no PyO3 dependency).

pub mod cycle;
pub mod decomposition;
pub mod filters;
pub mod heavy_kalman;
pub mod higher_order;
pub mod kalman;
pub mod prediction;
pub mod regime;
pub mod spectral;
pub mod spectral_heavy;
pub mod spectral_missing;
pub mod timefreq;
pub mod timefreq_extras;
pub mod wavelet;
pub mod wavelet_extra;
