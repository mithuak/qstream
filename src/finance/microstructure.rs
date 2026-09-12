//! Market microstructure bid-ask spread estimators from OHLC / quote data.

use crate::core::moments::RollingMean;
use crate::core::ring::RingBuffer;
use crate::core::traits::{HlcIndicator, PairIndicator};

/// Corwin-Schultz (2012) high-low bid-ask spread estimator.
///
/// Uses two consecutive high/low bars. With
/// `beta = sum_j ln(H_j/L_j)^2`, `gamma = ln(max H / min L)^2`,
/// `denom = 3 - 2*sqrt(2)`,
/// `alpha = (sqrt(2*beta) - sqrt(beta))/denom - sqrt(gamma/denom)`,
/// the relative spread is `2*(e^alpha - 1)/(e^alpha + 1)` with `alpha` floored
/// at zero. Returns `None` until two bars are available.
#[derive(Clone, Debug)]
pub struct CorwinSchultz {
    highs: RingBuffer<f64>,
    lows: RingBuffer<f64>,
}

impl CorwinSchultz {
    pub fn new() -> Self {
        Self {
            highs: RingBuffer::new(2, 0.0),
            lows: RingBuffer::new(2, 0.0),
        }
    }
}

impl Default for CorwinSchultz {
    fn default() -> Self {
        Self::new()
    }
}

impl PairIndicator for CorwinSchultz {
    type Output = f64;
    fn update(&mut self, high: f64, low: f64) -> Option<f64> {
        self.highs.push(high);
        self.lows.push(low);
        if self.highs.len() < 2 {
            return None;
        }
        let h0 = *self.highs.get(0);
        let h1 = *self.highs.get(1);
        let l0 = *self.lows.get(0);
        let l1 = *self.lows.get(1);
        let b0 = (h0 / l0).ln();
        let b1 = (h1 / l1).ln();
        let beta = b0 * b0 + b1 * b1;
        let gmax = h0.max(h1);
        let gmin = l0.min(l1);
        let gl = (gmax / gmin).ln();
        let gamma = gl * gl;
        let denom = 3.0 - 2.0 * std::f64::consts::SQRT_2;
        let mut alpha = (2.0f64.sqrt() * beta.sqrt() - beta.sqrt()) / denom
            - (gamma / denom).sqrt();
        if alpha < 0.0 {
            alpha = 0.0;
        }
        let ea = alpha.exp();
        Some(2.0 * (ea - 1.0) / (ea + 1.0))
    }
    fn reset(&mut self) {
        self.highs.clear(0.0);
        self.lows.clear(0.0);
    }
}

/// Abdi-Ranaldo (2016/2017) closing-price spread estimator.
///
/// Estimates the relative bid-ask spread from the bid-ask bounce signature in
/// consecutive closing-price changes, capped by the mean relative high-low
/// range. With `dc_t = c_t - c_{t-1}` and bounce
/// `b_t = max(0, dc_t) * max(0, -dc_{t-1})`, over a rolling window:
/// `spread = min(2*sqrt(mean(b)) / mean(c), mean((h-l)/c))`.
///
/// Returns `None` until the window has at least three closes.
#[derive(Clone, Debug)]
pub struct AbdiRanaldo {
    closes: RingBuffer<f64>,
    bounce: RollingMean,
    relrange: RollingMean,
    mean_close: RollingMean,
}

impl AbdiRanaldo {
    pub fn new(period: usize) -> Self {
        assert!(period > 1, "period must be > 1");
        Self {
            closes: RingBuffer::new(period + 1, 0.0),
            bounce: RollingMean::new(period),
            relrange: RollingMean::new(period),
            mean_close: RollingMean::new(period),
        }
    }
}

impl HlcIndicator for AbdiRanaldo {
    type Output = f64;
    fn update(&mut self, high: f64, low: f64, close: f64) -> Option<f64> {
        // Need the previous two closes to form a bounce term.
        let prev1 = self.closes.last().copied();
        let prev2 = if self.closes.len() >= 2 {
            Some(*self.closes.get(self.closes.len() - 2))
        } else {
            None
        };
        self.closes.push(close);
        self.relrange.update((high - low) / close);
        self.mean_close.update(close);

        if let (Some(c1), Some(c2)) = (prev1, prev2) {
            let dc1 = c1 - c2; // older change
            let dc2 = close - c1; // newer change
            let b = (dc2.max(0.0)) * ((-dc1).max(0.0));
            self.bounce.update(b);
        }

        // Warm-up: require a full mean-close window and at least one bounce.
        if !self.mean_close.is_ready() || self.bounce.count() == 0 {
            return None;
        }
        let mc = self.mean_close.value();
        if mc <= 0.0 {
            return None;
        }
        let est = 2.0 * self.bounce.value().max(0.0).sqrt() / mc;
        Some(est.min(self.relrange.value().max(0.0)))
    }
    fn reset(&mut self) {
        self.closes.clear(0.0);
        self.bounce.reset();
        self.relrange.reset();
        self.mean_close.reset();
    }
}

/// Glosten-Milgrom (1985) style quote/adverse-selection tracker.
///
/// Primary output is the relative quoted spread `(ask-bid)/mid`. It also
/// estimates the price-impact coefficient `lambda` from the regression of
/// mid-price changes on signed trade direction (Hasbrouck/Glosten-Milgrom
/// adverse-selection component), exposed via [`GlostenMilgrom::price_impact`].
#[derive(Clone, Debug)]
pub struct GlostenMilgrom {
    prev_mid: f64,
    have_prev: bool,
    // online regression of delta_mid on signed direction
    n: f64,
    sx: f64,
    sy: f64,
    sxx: f64,
    sxy: f64,
    last_spread: f64,
}

impl GlostenMilgrom {
    pub fn new() -> Self {
        Self {
            prev_mid: 0.0,
            have_prev: false,
            n: 0.0,
            sx: 0.0,
            sy: 0.0,
            sxx: 0.0,
            sxy: 0.0,
            last_spread: 0.0,
        }
    }

    /// Update with a trade and the prevailing quote. Returns the relative
    /// quoted spread.
    pub fn update(&mut self, trade_price: f64, bid: f64, ask: f64) -> f64 {
        let mid = 0.5 * (bid + ask);
        let spread = if mid > 0.0 { (ask - bid) / mid } else { 0.0 };
        self.last_spread = spread;

        // Trade direction: +1 buy at/above ask, -1 sell at/below bid, else sign
        // relative to mid.
        let dir = if trade_price >= ask {
            1.0
        } else if trade_price <= bid {
            -1.0
        } else if trade_price > mid {
            1.0
        } else if trade_price < mid {
            -1.0
        } else {
            0.0
        };

        if self.have_prev {
            let dmid = mid - self.prev_mid;
            // accumulate regression dmid ~ lambda * dir
            self.n += 1.0;
            self.sx += dir;
            self.sy += dmid;
            self.sxx += dir * dir;
            self.sxy += dir * dmid;
        }
        self.prev_mid = mid;
        self.have_prev = true;
        spread
    }

    /// Estimated price-impact (adverse-selection) coefficient lambda.
    pub fn price_impact(&self) -> f64 {
        let denom = self.n * self.sxx - self.sx * self.sx;
        if denom.abs() < 1e-18 {
            0.0
        } else {
            (self.n * self.sxy - self.sx * self.sy) / denom
        }
    }

    #[inline]
    pub fn spread(&self) -> f64 {
        self.last_spread
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.n as usize
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for GlostenMilgrom {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corwin_schultz_needs_two_bars() {
        let mut cs = CorwinSchultz::new();
        assert!(cs.update(105.0, 100.0).is_none());
        let s = cs.update(108.0, 101.0);
        assert!(s.is_some());
        assert!(s.unwrap() >= 0.0);
    }

    #[test]
    fn glosten_milgrom_spread() {
        let mut gm = GlostenMilgrom::new();
        let s = gm.update(100.5, 100.0, 101.0);
        // relative spread = 1/100.5
        assert!((s - 1.0 / 100.5).abs() < 1e-12);
    }
}
