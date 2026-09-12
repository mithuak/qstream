//! Rolling moments, sums, variance and EWMA. All are O(1) per update after
//! initialization and perform no heap allocation in the hot path.

use super::ring::RingBuffer;

/// Running sum over a fixed window.
#[derive(Clone, Debug)]
pub struct RollingSum {
    buf: RingBuffer<f64>,
    sum: f64,
}

impl RollingSum {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self { buf: RingBuffer::new(period, 0.0), sum: 0.0 }
    }
    #[inline]
    pub fn update(&mut self, x: f64) -> f64 {
        if let Some(e) = self.buf.push(x) {
            self.sum -= e;
        }
        self.sum += x;
        self.sum
    }
    #[inline]
    pub fn value(&self) -> f64 {
        self.sum
    }
    #[inline]
    pub fn count(&self) -> usize {
        self.buf.len()
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.buf.is_full()
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.sum = 0.0;
    }
}

/// Running mean over a fixed window.
#[derive(Clone, Debug)]
pub struct RollingMean {
    inner: RollingSum,
}

impl RollingMean {
    pub fn new(period: usize) -> Self {
        Self { inner: RollingSum::new(period) }
    }
    #[inline]
    pub fn update(&mut self, x: f64) -> f64 {
        let s = self.inner.update(x);
        s / self.inner.count() as f64
    }
    #[inline]
    pub fn value(&self) -> f64 {
        self.inner.value() / self.inner.count() as f64
    }
    #[inline]
    pub fn count(&self) -> usize {
        self.inner.count()
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

/// Rolling variance (sample, Bessel-corrected) via sum and sum-of-squares with
/// eviction. O(1) per update.
#[derive(Clone, Debug)]
pub struct RollingVariance {
    buf: RingBuffer<f64>,
    s1: f64,
    s2: f64,
    sample: bool,
}

impl RollingVariance {
    pub fn new(period: usize) -> Self {
        Self::with_correction(period, true)
    }
    pub fn with_correction(period: usize, sample: bool) -> Self {
        assert!(period > 0, "period must be > 0");
        Self { buf: RingBuffer::new(period, 0.0), s1: 0.0, s2: 0.0, sample }
    }
    #[inline]
    pub fn update(&mut self, x: f64) -> f64 {
        if let Some(e) = self.buf.push(x) {
            self.s1 -= e;
            self.s2 -= e * e;
        }
        self.s1 += x;
        self.s2 += x * x;
        self.variance()
    }
    #[inline]
    pub fn variance(&self) -> f64 {
        let n = self.buf.len() as f64;
        if n < 2.0 {
            return 0.0;
        }
        let mean = self.s1 / n;
        let denom = if self.sample { n - 1.0 } else { n };
        ((self.s2 / n) - mean * mean) * n / denom
    }
    #[inline]
    pub fn std_dev(&self) -> f64 {
        self.variance().max(0.0).sqrt()
    }
    #[inline]
    pub fn mean(&self) -> f64 {
        let n = self.buf.len() as f64;
        if n == 0.0 {
            0.0
        } else {
            self.s1 / n
        }
    }
    #[inline]
    pub fn count(&self) -> usize {
        self.buf.len()
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.buf.is_full()
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.s1 = 0.0;
        self.s2 = 0.0;
    }
}

/// Rolling central moments up to the fourth order (mean, variance, skewness,
/// excess kurtosis). Maintains raw power sums with eviction; O(1) per update.
#[derive(Clone, Debug)]
pub struct RollingMoments {
    buf: RingBuffer<f64>,
    s1: f64,
    s2: f64,
    s3: f64,
    s4: f64,
}

impl RollingMoments {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be > 0");
        Self { buf: RingBuffer::new(period, 0.0), s1: 0.0, s2: 0.0, s3: 0.0, s4: 0.0 }
    }
    #[inline]
    pub fn update(&mut self, x: f64) {
        if let Some(e) = self.buf.push(x) {
            self.s1 -= e;
            self.s2 -= e * e;
            self.s3 -= e * e * e;
            let e4 = e * e;
            self.s4 -= e4 * e4;
        }
        self.s1 += x;
        let x2 = x * x;
        self.s2 += x2;
        self.s3 += x2 * x;
        self.s4 += x2 * x2;
    }
    #[inline]
    pub fn count(&self) -> usize {
        self.buf.len()
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.buf.is_full()
    }
    #[inline]
    pub fn mean(&self) -> f64 {
        let n = self.buf.len() as f64;
        if n == 0.0 {
            0.0
        } else {
            self.s1 / n
        }
    }
    /// Population variance (second central moment).
    #[inline]
    pub fn variance(&self) -> f64 {
        let n = self.buf.len() as f64;
        if n < 2.0 {
            return 0.0;
        }
        let m = self.s1 / n;
        (self.s2 / n - m * m).max(0.0)
    }
    /// Sample (Bessel-corrected) variance.
    #[inline]
    pub fn sample_variance(&self) -> f64 {
        let n = self.buf.len() as f64;
        if n < 2.0 {
            return 0.0;
        }
        self.variance() * n / (n - 1.0)
    }
    #[inline]
    pub fn std_dev(&self) -> f64 {
        self.sample_variance().sqrt()
    }
    /// Third central moment (population).
    #[inline]
    pub fn m3(&self) -> f64 {
        let n = self.buf.len() as f64;
        if n < 3.0 {
            return 0.0;
        }
        let m = self.s1 / n;
        self.s3 / n - 3.0 * m * (self.s2 / n) + 2.0 * m * m * m
    }
    /// Fourth central moment (population).
    #[inline]
    pub fn m4(&self) -> f64 {
        let n = self.buf.len() as f64;
        if n < 4.0 {
            return 0.0;
        }
        let m = self.s1 / n;
        let m2 = self.s2 / n;
        let m3 = self.s3 / n;
        let m4 = self.s4 / n;
        m4 - 4.0 * m * m3 + 6.0 * m * m * m2 - 3.0 * m * m * m * m
    }
    #[inline]
    pub fn skewness(&self) -> f64 {
        let n = self.buf.len() as f64;
        let v = self.variance();
        if n < 3.0 || v <= 0.0 {
            return 0.0;
        }
        self.m3() / v.powf(1.5)
    }
    #[inline]
    pub fn kurtosis(&self) -> f64 {
        let n = self.buf.len() as f64;
        let v = self.variance();
        if n < 4.0 || v <= 0.0 {
            return 0.0;
        }
        self.m4() / (v * v)
    }
    /// Excess kurtosis (kurtosis - 3).
    #[inline]
    pub fn excess_kurtosis(&self) -> f64 {
        self.kurtosis() - 3.0
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.s1 = 0.0;
        self.s2 = 0.0;
        self.s3 = 0.0;
        self.s4 = 0.0;
    }
}

/// Exponentially weighted moving average with optional bias correction.
#[derive(Clone, Debug)]
pub struct Ewma {
    alpha: f64,
    value: f64,
    count: u64,
    bias_correct: bool,
}

impl Ewma {
    pub fn new(alpha: f64) -> Self {
        Self::with_bias(alpha, false)
    }
    pub fn with_bias(alpha: f64, bias_correct: bool) -> Self {
        assert!(alpha > 0.0 && alpha <= 1.0, "alpha must be in (0, 1]");
        Self { alpha, value: 0.0, count: 0, bias_correct }
    }
    /// Convenience constructor from a span (period), as in `alpha = 2/(span+1)`.
    pub fn from_span(span: f64) -> Self {
        Self::new(2.0 / (span + 1.0))
    }
    #[inline]
    pub fn update(&mut self, x: f64) -> f64 {
        if self.count == 0 {
            self.value = x;
        } else {
            self.value = self.alpha * x + (1.0 - self.alpha) * self.value;
        }
        self.count += 1;
        self.value()
    }
    #[inline]
    pub fn value(&self) -> f64 {
        if self.bias_correct && self.count > 0 {
            let denom = 1.0 - (1.0 - self.alpha).powi(self.count as i32);
            if denom > 0.0 {
                return self.value / denom;
            }
        }
        self.value
    }
    #[inline]
    pub fn raw(&self) -> f64 {
        self.value
    }
    #[inline]
    pub fn count(&self) -> u64 {
        self.count
    }
    pub fn reset(&mut self) {
        self.value = 0.0;
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolling_mean_matches_manual() {
        let mut rm = RollingMean::new(3);
        rm.update(1.0);
        rm.update(2.0);
        let v = rm.update(3.0);
        assert!((v - 2.0).abs() < 1e-12);
        let v = rm.update(4.0);
        assert!((v - 3.0).abs() < 1e-12);
    }

    #[test]
    fn rolling_variance_matches_manual() {
        let mut rv = RollingVariance::new(4);
        for x in [1.0, 2.0, 3.0, 4.0] {
            rv.update(x);
        }
        // sample variance of [1,2,3,4] = 5/3
        assert!((rv.variance() - 5.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn ewma_recursion() {
        let mut e = Ewma::new(0.5);
        assert!((e.update(10.0) - 10.0).abs() < 1e-12);
        assert!((e.update(20.0) - 15.0).abs() < 1e-12);
        assert!((e.update(30.0) - 22.5).abs() < 1e-12);
    }

    #[test]
    fn moments_skew_kurtosis() {
        let mut m = RollingMoments::new(5);
        for x in [1.0, 2.0, 3.0, 4.0, 5.0] {
            m.update(x);
        }
        assert!((m.mean() - 3.0).abs() < 1e-12);
        assert!(m.skewness().abs() < 1e-9); // symmetric
    }
}
