//! Rolling quantiles and tail accumulation for historical VaR / CVaR.
//! Tier B: fixed-capacity ring buffer, quantiles computed on demand into a
//! reusable scratch vector (sorted copy).

use super::ring::RingBuffer;

/// Rolling empirical quantile (linear interpolation, "type-7" like NumPy).
#[derive(Clone, Debug)]
pub struct RollingQuantile {
    buf: RingBuffer<f64>,
    scratch: Vec<f64>,
}

impl RollingQuantile {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self { buf: RingBuffer::new(period, 0.0), scratch: Vec::with_capacity(period) }
    }
    #[inline]
    pub fn update(&mut self, x: f64) {
        self.buf.push(x);
    }
    #[inline]
    pub fn count(&self) -> usize {
        self.buf.len()
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.buf.is_full()
    }
    /// Quantile `q in [0,1]` of the retained window.
    pub fn quantile(&mut self, q: f64) -> f64 {
        let n = self.buf.len();
        if n == 0 {
            return f64::NAN;
        }
        self.buf.fill_vec(&mut self.scratch);
        self.scratch.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if n == 1 {
            return self.scratch[0];
        }
        let qc = q.clamp(0.0, 1.0);
        let h = (n as f64 - 1.0) * qc;
        let lo = h.floor() as usize;
        let hi = h.ceil() as usize;
        if lo == hi {
            self.scratch[lo]
        } else {
            self.scratch[lo] + (h - lo as f64) * (self.scratch[hi] - self.scratch[lo])
        }
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.scratch.clear();
    }
}

/// Rolling tail accumulator for historical Value-at-Risk and Conditional VaR
/// (expected shortfall) over a return window.
#[derive(Clone, Debug)]
pub struct TailAccumulator {
    buf: RingBuffer<f64>,
    scratch: Vec<f64>,
}

impl TailAccumulator {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self { buf: RingBuffer::new(period, 0.0), scratch: Vec::with_capacity(period) }
    }
    #[inline]
    pub fn update(&mut self, r: f64) {
        self.buf.push(r);
    }
    #[inline]
    pub fn count(&self) -> usize {
        self.buf.len()
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.buf.is_full()
    }

    fn sorted(&mut self) -> usize {
        self.buf.fill_vec(&mut self.scratch);
        self.scratch.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.scratch.len()
    }

    /// Historical VaR at tail probability `p` (e.g. 0.05). Returned as a
    /// positive loss magnitude: `-quantile(returns, p)`.
    pub fn var(&mut self, p: f64) -> f64 {
        let n = self.sorted();
        if n == 0 {
            return f64::NAN;
        }
        let q = quantile_sorted(&self.scratch, p.clamp(0.0, 1.0));
        -q
    }

    /// Conditional VaR / Expected Shortfall at tail probability `p`:
    /// mean of losses at least as bad as the VaR, returned positive.
    pub fn cvar(&mut self, p: f64) -> f64 {
        let n = self.sorted();
        if n == 0 {
            return f64::NAN;
        }
        let pc = p.clamp(0.0, 1.0);
        // Number of observations in the tail (at least 1).
        let k = ((pc * n as f64).ceil() as usize).max(1).min(n);
        let mut acc = 0.0;
        for i in 0..k {
            acc += self.scratch[i];
        }
        -(acc / k as f64)
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.scratch.clear();
    }
}

/// Type-7 quantile of an already-sorted slice.
pub fn quantile_sorted(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return sorted[0];
    }
    let h = (n as f64 - 1.0) * q.clamp(0.0, 1.0);
    let lo = h.floor() as usize;
    let hi = h.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        sorted[lo] + (h - lo as f64) * (sorted[hi] - sorted[lo])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolling_quantile_median() {
        let mut q = RollingQuantile::new(5);
        for x in [5.0, 1.0, 3.0, 2.0, 4.0] {
            q.update(x);
        }
        assert!((q.quantile(0.5) - 3.0).abs() < 1e-12);
    }

    #[test]
    fn tail_var_cvar() {
        let mut t = TailAccumulator::new(10);
        for x in [-0.10, -0.05, -0.03, -0.01, 0.0, 0.01, 0.02, 0.03, 0.04, 0.05] {
            t.update(x);
        }
        // 10% VaR ~ -quantile(0.1). Worst is -0.10.
        let v = t.var(0.1);
        assert!(v > 0.0);
        let c = t.cvar(0.2);
        assert!(c >= v - 1e-9);
    }
}
