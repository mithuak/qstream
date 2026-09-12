//! Core indicator traits. Multiple shapes rather than forcing everything into a
//! single scalar interface (design section 5).

/// Streaming indicator consuming one scalar per tick.
pub trait ScalarIndicator {
    type Output;
    fn update(&mut self, value: f64) -> Option<Self::Output>;
    fn reset(&mut self);
}

/// Streaming indicator consuming one OHLC bar per tick.
pub trait OhlcIndicator {
    type Output;
    fn update(&mut self, open: f64, high: f64, low: f64, close: f64) -> Option<Self::Output>;
    fn reset(&mut self);
}

/// Streaming indicator consuming a pair (asset, benchmark) per tick.
pub trait PairIndicator {
    type Output;
    fn update(&mut self, x: f64, y: f64) -> Option<Self::Output>;
    fn reset(&mut self);
}

/// Streaming indicator consuming a high/low/close bar (microstructure).
pub trait HlcIndicator {
    type Output;
    fn update(&mut self, high: f64, low: f64, close: f64) -> Option<Self::Output>;
    fn reset(&mut self);
}
