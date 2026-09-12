//! Drawdown state: running peak, current/max drawdown, and a bounded window of
//! drawdown values for Conditional Drawdown-at-Risk (CDaR).

use super::quantile::quantile_sorted;
use super::ring::RingBuffer;

/// Tracks drawdown from a wealth curve. Feed either prices/levels
/// (`update_level`) or simple returns (`update_return`).
#[derive(Clone, Debug)]
pub struct DrawdownState {
    level: f64,
    peak: f64,
    max_drawdown: f64,
    initialized: bool,
    /// Bounded window of observed drawdown magnitudes for CDaR.
    dd_window: RingBuffer<f64>,
    scratch: Vec<f64>,
    min_level: f64,
}

impl DrawdownState {
    /// `window` bounds the CDaR lookback (drawdown history).
    pub fn new(window: usize) -> Self {
        assert!(window > 0, "window must be > 0");
        Self {
            level: 0.0,
            peak: f64::NEG_INFINITY,
            max_drawdown: 0.0,
            initialized: false,
            dd_window: RingBuffer::new(window, 0.0),
            scratch: Vec::with_capacity(window),
            min_level: f64::NEG_INFINITY,
        }
    }

    /// Update from a price/wealth level. Returns the current drawdown.
    pub fn update_level(&mut self, level: f64) -> f64 {
        if !self.initialized {
            self.level = level;
            self.peak = level;
            self.min_level = level;
            self.initialized = true;
        } else {
            self.level = level;
            if level > self.peak {
                self.peak = level;
            }
            if level < self.min_level {
                self.min_level = level;
            }
        }
        let dd = if self.peak > 0.0 {
            (self.peak - self.level) / self.peak
        } else {
            0.0
        };
        let dd = dd.max(0.0);
        if dd > self.max_drawdown {
            self.max_drawdown = dd;
        }
        self.dd_window.push(dd);
        dd
    }

    /// Update from a simple return: wealth grows by `(1 + r)`.
    pub fn update_return(&mut self, r: f64) -> f64 {
        let base = if self.initialized { self.level } else { 1.0 };
        let level = base * (1.0 + r);
        self.update_level(level)
    }

    #[inline]
    pub fn current(&self) -> f64 {
        if self.peak > 0.0 {
            ((self.peak - self.level) / self.peak).max(0.0)
        } else {
            0.0
        }
    }
    #[inline]
    pub fn max_drawdown(&self) -> f64 {
        self.max_drawdown
    }
    #[inline]
    pub fn peak(&self) -> f64 {
        self.peak
    }

    /// Conditional Drawdown-at-Risk at confidence `alpha` (e.g. 0.95):
    /// the mean of the worst `(1 - alpha)` fraction of observed drawdowns.
    pub fn cdar(&mut self, alpha: f64) -> f64 {
        self.dd_window.fill_vec(&mut self.scratch);
        let n = self.scratch.len();
        if n == 0 {
            return f64::NAN;
        }
        self.scratch.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // Tail = largest drawdowns above the alpha quantile.
        let a = alpha.clamp(0.0, 1.0);
        let threshold = quantile_sorted(&self.scratch, a);
        let mut acc = 0.0;
        let mut cnt = 0usize;
        for &d in self.scratch.iter() {
            if d >= threshold {
                acc += d;
                cnt += 1;
            }
        }
        if cnt == 0 {
            threshold
        } else {
            acc / cnt as f64
        }
    }

    pub fn reset(&mut self) {
        self.level = 0.0;
        self.peak = f64::NEG_INFINITY;
        self.min_level = f64::NEG_INFINITY;
        self.max_drawdown = 0.0;
        self.initialized = false;
        self.dd_window.clear(0.0);
        self.scratch.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawdown_tracks_peak() {
        let mut d = DrawdownState::new(64);
        d.update_level(100.0);
        d.update_level(120.0);
        let dd = d.update_level(60.0);
        assert!((dd - 0.5).abs() < 1e-12); // (120-60)/120
        assert!((d.max_drawdown() - 0.5).abs() < 1e-12);
    }
}
