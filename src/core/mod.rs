//! Core reusable primitives shared by finance and signal modules.

pub mod covariance;
pub mod drawdown;
pub mod fft;
pub mod matrix;
pub mod moments;
pub mod num;
pub mod quantile;
pub mod regression;
pub mod ring;
pub mod traits;
pub mod window;

pub use covariance::{EwmaCovariance, RollingCovariance};
pub use drawdown::DrawdownState;
pub use fft::{FftPlan, FftPlanCache};
pub use matrix::DMat;
pub use moments::{Ewma, RollingMean, RollingMoments, RollingSum, RollingVariance};
pub use num::Complex;
pub use quantile::{quantile_sorted, RollingQuantile, TailAccumulator};
pub use regression::{OnlineSimpleRegression, RecursiveLeastSquares, RollingSimpleRegression};
pub use ring::RingBuffer;
pub use traits::{HlcIndicator, OhlcIndicator, PairIndicator, ScalarIndicator};
pub use window::{dpss, fill_window, WindowKind};
