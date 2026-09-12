//! Linear-prediction primitives: autocorrelation, Levinson-Durbin recursion,
//! Burg AR estimation, a streaming LPC predictor, and a lattice
//! prediction-error filter.

use crate::core::ring::RingBuffer;

/// Result of a prediction update.
#[derive(Clone, Debug)]
pub struct PredictionResult {
    pub prediction: f64,
    pub coefficients: Vec<f64>,
}

/// Sample autocorrelation (mean-removed) up to `order`. Returns `order+1`
/// values, `acf[0]` being the variance-like term.
pub fn autocorrelation(x: &[f64], order: usize) -> Vec<f64> {
    let n = x.len();
    let mut acf = vec![0.0; order + 1];
    if n == 0 {
        return acf;
    }
    let mean = x.iter().sum::<f64>() / n as f64;
    for lag in 0..=order.min(n - 1) {
        let mut s = 0.0;
        for t in lag..n {
            s += (x[t] - mean) * (x[t - lag] - mean);
        }
        acf[lag] = s / n as f64;
    }
    acf
}

/// Levinson-Durbin recursion. Given autocorrelation `acf[0..=order]`, returns
/// `(ar_coefficients, prediction_error_variance, reflection_coefficients)`.
/// AR model: `x_t = sum_{i=1..order} a_i x_{t-i} + e_t`.
pub fn levinson_durbin(acf: &[f64], order: usize) -> (Vec<f64>, f64, Vec<f64>) {
    let mut a = vec![0.0; order + 1];
    let mut k = vec![0.0; order + 1];
    let mut e = if acf.is_empty() { 0.0 } else { acf[0] };
    if e <= 0.0 {
        return (vec![0.0; order], 0.0, vec![0.0; order]);
    }
    for m in 1..=order {
        if m >= acf.len() {
            break;
        }
        let mut lambda = acf[m];
        for i in 1..m {
            lambda -= a[i] * acf[m - i];
        }
        let km = lambda / e;
        k[m] = km;
        // Update coefficients.
        let mut new_a = a.clone();
        for i in 1..m {
            new_a[i] = a[i] - km * a[m - i];
        }
        new_a[m] = km;
        a = new_a;
        e *= 1.0 - km * km;
        if e <= 0.0 {
            break;
        }
    }
    (a[1..=order].to_vec(), e.max(0.0), k[1..=order].to_vec())
}

/// Burg's method for AR coefficient estimation. Returns
/// `(ar_coefficients, reflection_coefficients)`.
pub fn burg_ar(x: &[f64], order: usize) -> (Vec<f64>, Vec<f64>) {
    let n = x.len();
    let mut a = vec![0.0; order + 1];
    let mut k = vec![0.0; order + 1];
    if n <= order || n == 0 {
        return (vec![0.0; order], vec![0.0; order]);
    }
    let mut ef = x.to_vec();
    let mut eb = x.to_vec();
    for m in 1..=order {
        let mut num = 0.0;
        let mut den = 0.0;
        for t in m..n {
            num += ef[t] * eb[t - 1];
            den += ef[t] * ef[t] + eb[t - 1] * eb[t - 1];
        }
        let km = if den > 1e-18 { 2.0 * num / den } else { 0.0 };
        k[m] = km;
        let mut new_a = a.clone();
        for i in 1..m {
            new_a[i] = a[i] - km * a[m - i];
        }
        new_a[m] = km;
        a = new_a;
        // Update forward/backward errors.
        let mut ef_new = ef.clone();
        let mut eb_new = eb.clone();
        for t in m..n {
            ef_new[t] = ef[t] - km * eb[t - 1];
            eb_new[t] = eb[t - 1] - km * ef[t];
        }
        ef = ef_new;
        eb = eb_new;
    }
    (a[1..=order].to_vec(), k[1..=order].to_vec())
}

/// Streaming LPC / Levinson-Durbin predictor over a rolling window. AR
/// coefficients are recomputed every `update_every` samples (Tier C cadence);
/// the one-step prediction is produced every tick once warm.
#[derive(Clone, Debug)]
pub struct LpcPredictor {
    buf: RingBuffer<f64>,
    order: usize,
    update_every: usize,
    since: usize,
    coeffs: Vec<f64>,
    data: Vec<f64>,
}

impl LpcPredictor {
    pub fn new(window: usize, order: usize, update_every: usize) -> Self {
        assert!(window > order + 1, "window must exceed order+1");
        assert!(order > 0 && update_every > 0);
        Self {
            buf: RingBuffer::new(window, 0.0),
            order,
            update_every,
            since: 0,
            coeffs: vec![0.0; order],
            data: Vec::with_capacity(window),
        }
    }
    pub fn update(&mut self, x: f64) -> Option<PredictionResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.buf.fill_vec(&mut self.data);
        let n = self.data.len();
        // Recompute coefficients on cadence.
        self.since += 1;
        if self.since >= self.update_every {
            self.since = 0;
            let acf = autocorrelation(&self.data, self.order);
            let (c, _, _) = levinson_durbin(&acf, self.order);
            self.coeffs = c;
        }
        // One-step prediction for the current sample from the previous ones.
        let mut pred = 0.0;
        for i in 0..self.order {
            // data[n-1] is the current sample; use earlier ones.
            let idx = n as isize - 2 - i as isize;
            if idx >= 0 {
                pred += self.coeffs[i] * self.data[idx as usize];
            }
        }
        Some(PredictionResult { prediction: pred, coefficients: self.coeffs.clone() })
    }
    #[inline]
    pub fn coefficients(&self) -> &[f64] {
        &self.coeffs
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.coeffs.iter_mut().for_each(|c| *c = 0.0);
        self.since = 0;
        self.data.clear();
    }
}

/// Lattice prediction-error filter. Uses reflection coefficients (recomputed on
/// a cadence from a Burg estimate) and runs the forward/backward lattice
/// recursion, outputting the order-`p` forward prediction error (the whitened
/// residual). Equivalent to `x_t - sum a_i x_{t-i}` for the AR model.
#[derive(Clone, Debug)]
pub struct LatticePredictionErrorFilter {
    order: usize,
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    refl: Vec<f64>,
    bprev: Vec<f64>,
    update_every: usize,
    since: usize,
}

impl LatticePredictionErrorFilter {
    pub fn new(window: usize, order: usize, update_every: usize) -> Self {
        assert!(window > order + 1, "window must exceed order+1");
        Self {
            order,
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            refl: vec![0.0; order],
            bprev: vec![0.0; order],
            update_every,
            since: 0,
        }
    }
    pub fn update(&mut self, x: f64) -> Option<f64> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since >= self.update_every {
            self.since = 0;
            self.buf.fill_vec(&mut self.data);
            let (_, k) = burg_ar(&self.data, self.order);
            self.refl = k;
        }
        // Lattice recursion.
        let mut f = x;
        let mut bcur = vec![0.0; self.order];
        bcur[0] = x; // b_0(n)
        for m in 1..=self.order {
            let f_prev = f; // f_{m-1}(n)
            let b_prev = self.bprev[m - 1]; // b_{m-1}(n-1)
            let km = self.refl[m - 1];
            let f_m = f_prev - km * b_prev;
            if m < self.order {
                bcur[m] = b_prev - km * f_prev;
            }
            f = f_m;
        }
        self.bprev = bcur;
        Some(f)
    }
    #[inline]
    pub fn reflection_coefficients(&self) -> &[f64] {
        &self.refl
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.refl.iter_mut().for_each(|c| *c = 0.0);
        self.bprev.iter_mut().for_each(|c| *c = 0.0);
        self.since = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levinson_recovers_ar1() {
        // Autocorrelation of AR(1) with a=0.9: r_k = 0.9^k.
        let order = 4;
        let acf: Vec<f64> = (0..=order).map(|k| 0.9f64.powi(k as i32)).collect();
        let (a, e, _) = levinson_durbin(&acf, order);
        assert!((a[0] - 0.9).abs() < 1e-6);
        for i in 1..order {
            assert!(a[i].abs() < 1e-6);
        }
        assert!(e > 0.0);
    }

    #[test]
    fn burg_ar1() {
        // AR(1) driven by deterministic pseudo-white noise (LCG).
        let mut seed: u64 = 0x2545F4914F6CDD1D;
        let mut noise = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((seed >> 33) as f64) / (1u64 << 31) as f64 - 0.5
        };
        let mut x = vec![0.0];
        for i in 1..500 {
            x.push(0.8 * x[i - 1] + noise());
        }
        let (a, _) = burg_ar(&x, 1);
        assert!((a[0] - 0.8).abs() < 0.05, "burg a1 = {}", a[0]);
    }

    #[test]
    fn lattice_pef_whitens_predictable_signal() {
        // A single sinusoid is well modeled by AR(2); the lattice prediction
        // error should be far smaller in magnitude than the signal itself.
        let order = 2;
        let mut lat = LatticePredictionErrorFilter::new(64, order, 8);
        let mut sig_sq = 0.0;
        let mut err_sq = 0.0;
        let mut cnt = 0.0;
        for i in 0..300 {
            let x = (2.0 * std::f64::consts::PI * 0.05 * i as f64).sin();
            if let Some(e) = lat.update(x) {
                // Skip the first samples after warm-up while coefficients adapt.
                if i > 120 {
                    sig_sq += x * x;
                    err_sq += e * e;
                    cnt += 1.0;
                }
            }
        }
        assert!(cnt > 0.0);
        let sig_rms = (sig_sq / cnt).sqrt();
        let err_rms = (err_sq / cnt).sqrt();
        assert!(err_rms < 0.5 * sig_rms, "PEF should whiten: {} vs {}", err_rms, sig_rms);
    }
}
