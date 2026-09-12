//! Regime / energy features: Page-Hinkley change detector, Teager-Kaiser energy
//! operator, and zero-crossing rate.

use crate::core::ring::RingBuffer;
use crate::core::traits::ScalarIndicator;

/// Result of a change-detection update.
#[derive(Clone, Copy, Debug)]
pub struct ChangeResult {
    pub changed: bool,
    pub score: f64,
}

/// Page-Hinkley change detector for an upward shift in the running mean.
/// Maintains the cumulative deviation `m_T`, its running minimum `M_T`, and
/// raises a change when `m_T - M_T > threshold`. O(1) recursive, Tier A.
#[derive(Clone, Debug)]
pub struct PageHinkley {
    delta: f64,
    threshold: f64,
    mean: f64,
    n: f64,
    sum: f64,
    min: f64,
}

impl PageHinkley {
    /// `delta` is the minimum magnitude of change to accumulate; `threshold`
    /// (lambda) triggers the alarm.
    pub fn new(delta: f64, threshold: f64) -> Self {
        Self { delta, threshold, mean: 0.0, n: 0.0, sum: 0.0, min: 0.0 }
    }
    pub fn update(&mut self, x: f64) -> ChangeResult {
        self.n += 1.0;
        self.mean += (x - self.mean) / self.n;
        self.sum += x - self.mean - self.delta;
        if self.sum < self.min {
            self.min = self.sum;
        }
        let ph = self.sum - self.min;
        ChangeResult { changed: ph > self.threshold, score: ph }
    }
    #[inline]
    pub fn statistic(&self) -> f64 {
        self.sum - self.min
    }
    pub fn reset(&mut self) {
        self.mean = 0.0;
        self.n = 0.0;
        self.sum = 0.0;
        self.min = 0.0;
    }
}

/// Teager-Kaiser energy operator: `psi[x_n] = x_n^2 - x_{n-1} x_{n+1}`.
/// Reported for the center sample with a one-step delay. O(1), Tier A.
#[derive(Clone, Debug)]
pub struct TeagerKaiser {
    x1: f64,
    x2: f64,
    have: u8,
}

impl TeagerKaiser {
    pub fn new() -> Self {
        Self { x1: 0.0, x2: 0.0, have: 0 }
    }
}

impl Default for TeagerKaiser {
    fn default() -> Self {
        Self::new()
    }
}

impl ScalarIndicator for TeagerKaiser {
    type Output = f64;
    fn update(&mut self, value: f64) -> Option<f64> {
        let out = match self.have {
            0 => {
                self.have = 1;
                None
            }
            1 => {
                self.have = 2;
                None
            }
            _ => {
                // energy for the middle sample x1: x1^2 - x2*value
                Some(self.x1 * self.x1 - self.x2 * value)
            }
        };
        self.x2 = self.x1;
        self.x1 = value;
        out
    }
    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.have = 0;
    }
}

/// Zero-crossing rate over a rolling window, relative to a centering level.
/// `rate = crossings / (window - 1)`. Tier B (bounded window).
#[derive(Clone, Debug)]
pub struct ZeroCrossingRate {
    buf: RingBuffer<f64>,
    threshold: f64,
}

impl ZeroCrossingRate {
    pub fn new(window: usize, threshold: f64) -> Self {
        assert!(window > 1, "window must be > 1");
        Self { buf: RingBuffer::new(window, 0.0), threshold }
    }
    pub fn update(&mut self, x: f64) -> Option<f64> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        let n = self.buf.len();
        let mut crossings = 0usize;
        let mut prev = (*self.buf.get(0) - self.threshold).signum();
        for i in 1..n {
            let cur = (*self.buf.get(i) - self.threshold).signum();
            if cur != 0.0 && prev != 0.0 && cur != prev {
                crossings += 1;
            }
            if cur != 0.0 {
                prev = cur;
            }
        }
        Some(crossings as f64 / (n - 1) as f64)
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_hinkley_detects_shift() {
        let mut ph = PageHinkley::new(0.005, 1.0);
        for _ in 0..50 {
            ph.update(0.0);
        }
        let mut fired = false;
        for _ in 0..200 {
            if ph.update(1.0).changed {
                fired = true;
            }
        }
        assert!(fired);
    }

    #[test]
    fn teager_kaiser_sinusoid() {
        let mut tk = TeagerKaiser::new();
        let mut last = None;
        for i in 0..10 {
            let x = (i as f64 * 0.5).sin();
            last = tk.update(x);
        }
        // Energy of a sinusoid under TKEO is ~ sin^2(omega) * A^2 >= 0.
        assert!(last.unwrap() > 0.0);
    }

    #[test]
    fn zero_crossing_alternating() {
        let mut zc = ZeroCrossingRate::new(5, 0.0);
        zc.update(1.0);
        zc.update(-1.0);
        zc.update(1.0);
        zc.update(-1.0);
        let r = zc.update(1.0).unwrap();
        // 4 crossings over 4 intervals = 1.0
        assert!((r - 1.0).abs() < 1e-12);
    }
}
