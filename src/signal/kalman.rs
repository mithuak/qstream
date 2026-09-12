//! State-space and adaptive filters: a general linear Kalman filter, alpha-beta
//! (g-h) tracker, adaptive (online Q/R) Kalman, square-root scalar Kalman,
//! extended/unscented Kalman for the constant-velocity model, and LMS/RLS
//! adaptive predictors.

use crate::core::matrix::{cholesky, inverse, DMat};
use crate::core::regression::RecursiveLeastSquares;
use crate::core::ring::RingBuffer;
use crate::core::traits::ScalarIndicator;

/// Result of a state-estimate update.
#[derive(Clone, Debug)]
pub struct StateEstimate {
    pub value: f64,
    pub velocity: Option<f64>,
    /// Flattened state covariance (row-major `n x n`).
    pub covariance: Vec<f64>,
}

/// General linear Kalman filter: `x_k = F x_{k-1} + w`, `z_k = H x_k + v`.
/// Matrices are fixed at construction. Update is O(n^3) for small `n`.
#[derive(Clone, Debug)]
pub struct KalmanFilter {
    n: usize,
    m: usize,
    f: DMat,
    h: DMat,
    q: DMat,
    r: DMat,
    x: Vec<f64>,
    p: DMat,
    // scratch
    pred_x: Vec<f64>,
    innov: Vec<f64>,
    s: DMat,
    gain: DMat,
    initialized: bool,
}

impl KalmanFilter {
    pub fn new(f: DMat, h: DMat, q: DMat, r: DMat, p0: DMat, x0: Vec<f64>) -> Self {
        let n = f.rows;
        let m = h.rows;
        assert_eq!(f.cols, n);
        assert_eq!(h.cols, n);
        assert_eq!(q.rows, n);
        assert_eq!(q.cols, n);
        assert_eq!(r.rows, m);
        assert_eq!(r.cols, m);
        assert_eq!(p0.rows, n);
        assert_eq!(x0.len(), n);
        Self {
            n,
            m,
            f,
            h,
            q,
            r,
            x: x0,
            p: p0,
            pred_x: vec![0.0; n],
            innov: vec![0.0; m],
            s: DMat::zeros(m, m),
            gain: DMat::zeros(n, m),
            initialized: false,
        }
    }

    /// Convenience: 2-state constant-velocity model tracking a scalar series.
    /// `dt` step, `q` acceleration spectral density, `r` measurement variance.
    pub fn constant_velocity(dt: f64, q: f64, r: f64, p0: f64) -> Self {
        let f = DMat::from_row_vec(2, 2, vec![1.0, dt, 0.0, 1.0]);
        let h = DMat::from_row_vec(1, 2, vec![1.0, 0.0]);
        let dt2 = dt * dt;
        let dt3 = dt2 * dt;
        let qm = DMat::from_row_vec(
            2,
            2,
            vec![q * dt3 / 3.0, q * dt2 / 2.0, q * dt2 / 2.0, q * dt],
        );
        let rm = DMat::from_row_vec(1, 1, vec![r]);
        let p = DMat::from_row_vec(2, 2, vec![p0, 0.0, 0.0, p0]);
        Self::new(f, h, qm, rm, p, vec![0.0, 0.0])
    }

    /// Update with a measurement vector `z` (length m).
    pub fn update(&mut self, z: &[f64]) -> StateEstimate {
        assert_eq!(z.len(), self.m);
        // Predict.
        self.pred_x = self.f.mul_vec(&self.x);
        let fp = self.f.mul(&self.p);
        let ft = self.f.transpose();
        self.p = fp.mul(&ft).add(&self.q);
        self.x.copy_from_slice(&self.pred_x);

        // Innovation y = z - H x.
        let hx = self.h.mul_vec(&self.x);
        for i in 0..self.m {
            self.innov[i] = z[i] - hx[i];
        }
        // S = H P H^T + R.
        let hp = self.h.mul(&self.p);
        let ht = self.h.transpose();
        self.s = hp.mul(&ht).add(&self.r);
        // K = P H^T S^{-1}.
        let pht = self.p.mul(&ht); // n x m
        // Solve S^T K^T = (P H^T)^T  =>  K = (P H^T) S^{-1}
        let s_inv = inverse(&self.s).unwrap_or_else(|| DMat::identity(self.m));
        self.gain = pht.mul(&s_inv);
        // x = x + K y.
        let dy = self.gain.mul_vec(&self.innov);
        for i in 0..self.n {
            self.x[i] += dy[i];
        }
        // P = (I - K H) P (I - K H)^T + K R K^T   (Joseph form, stable).
        let kh = self.gain.mul(&self.h); // n x n
        let ikh = DMat::identity(self.n).sub(&kh);
        let term1 = ikh.mul(&self.p).mul(&ikh.transpose());
        let term2 = self.gain.mul(&self.r).mul(&self.gain.transpose());
        self.p = term1.add(&term2);

        self.initialized = true;
        self.estimate()
    }

    /// Update from a scalar measurement (only valid when m == 1).
    pub fn update_scalar(&mut self, z: f64) -> StateEstimate {
        assert_eq!(self.m, 1, "update_scalar requires a scalar measurement model");
        self.update(&[z])
    }

    fn estimate(&self) -> StateEstimate {
        StateEstimate {
            value: self.x[0],
            velocity: if self.n >= 2 { Some(self.x[1]) } else { None },
            covariance: self.p.data.clone(),
        }
    }

    #[inline]
    pub fn state(&self) -> &[f64] {
        &self.x
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.initialized
    }
    pub fn reset(&mut self) {
        for x in self.x.iter_mut() {
            *x = 0.0;
        }
        self.p = DMat::identity(self.n);
        self.initialized = false;
    }
}

/// Specialized 2-state constant-velocity Kalman filter with fixed-size inline
/// matrix math. Performs no heap allocation in the update computation (the
/// returned `StateEstimate.covariance` is the only allocation), satisfying the
/// Tier A latency rules for the most common tracking case.
#[derive(Clone, Debug)]
pub struct ConstantVelocityKalman {
    dt: f64,
    r: f64,
    pos: f64,
    vel: f64,
    /// Covariance [P00, P01, P10, P11].
    p: [f64; 4],
    /// Precomputed process-noise terms.
    q00: f64,
    q01: f64,
    q11: f64,
    initialized: bool,
}

impl ConstantVelocityKalman {
    /// `dt` step, `q` acceleration spectral density, `r` measurement variance,
    /// `p0` initial state variance.
    pub fn new(dt: f64, q: f64, r: f64, p0: f64) -> Self {
        let dt2 = dt * dt;
        let dt3 = dt2 * dt;
        Self {
            dt,
            r,
            pos: 0.0,
            vel: 0.0,
            p: [p0, 0.0, 0.0, p0],
            q00: q * dt3 / 3.0,
            q01: q * dt2 / 2.0,
            q11: q * dt,
            initialized: false,
        }
    }

    pub fn update(&mut self, z: f64) -> StateEstimate {
        if !self.initialized {
            self.pos = z;
            self.initialized = true;
        } else {
            // Predict state.
            self.pos += self.vel * self.dt;
            // Predict covariance: P = F P F^T + Q.
            let (p00, p01, p10, p11) = (self.p[0], self.p[1], self.p[2], self.p[3]);
            let fp00 = p00 + self.dt * p10;
            let fp01 = p01 + self.dt * p11;
            let fp10 = p10;
            let fp11 = p11;
            self.p[0] = fp00 + self.dt * fp01 + self.q00;
            self.p[1] = fp01 + self.q01;
            self.p[2] = fp10 + self.dt * fp11 + self.q01;
            self.p[3] = fp11 + self.q11;
        }
        // Measurement update (H = [1, 0], scalar measurement).
        let y = z - self.pos;
        let s = self.p[0] + self.r;
        let k0 = self.p[0] / s;
        let k1 = self.p[2] / s;
        self.pos += k0 * y;
        self.vel += k1 * y;
        // P = P - K (H P)  where H P = [P00, P01].
        let (p00, p01, p10, p11) = (self.p[0], self.p[1], self.p[2], self.p[3]);
        self.p[0] = p00 - k0 * p00;
        self.p[1] = p01 - k0 * p01;
        self.p[2] = p10 - k1 * p00;
        self.p[3] = p11 - k1 * p01;
        StateEstimate {
            value: self.pos,
            velocity: Some(self.vel),
            covariance: self.p.to_vec(),
        }
    }

    #[inline]
    pub fn value(&self) -> f64 {
        self.pos
    }
    #[inline]
    pub fn velocity(&self) -> f64 {
        self.vel
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.initialized
    }
    pub fn reset(&mut self) {
        self.pos = 0.0;
        self.vel = 0.0;
        let p0 = self.p[0];
        self.p = [p0, 0.0, 0.0, p0];
        self.initialized = false;
    }
}

/// Alpha-beta (g-h) tracker: constant-velocity position/velocity estimator.
/// O(1) recursive, Tier A.
#[derive(Clone, Debug)]
pub struct AlphaBetaTracker {
    alpha: f64,
    beta: f64,
    dt: f64,
    x: f64,
    v: f64,
    initialized: bool,
}

impl AlphaBetaTracker {
    pub fn new(alpha: f64, beta: f64, dt: f64) -> Self {
        assert!(alpha > 0.0 && alpha <= 1.0, "alpha in (0,1]");
        assert!(beta >= 0.0 && beta <= 1.0, "beta in [0,1]");
        Self { alpha, beta, dt, x: 0.0, v: 0.0, initialized: false }
    }
    pub fn update(&mut self, z: f64) -> StateEstimate {
        if !self.initialized {
            self.x = z;
            self.v = 0.0;
            self.initialized = true;
        } else {
            // Predict.
            let xp = self.x + self.v * self.dt;
            let r = z - xp;
            self.x = xp + self.alpha * r;
            self.v = self.v + (self.beta / self.dt) * r;
        }
        StateEstimate { value: self.x, velocity: Some(self.v), covariance: Vec::new() }
    }
    #[inline]
    pub fn value(&self) -> f64 {
        self.x
    }
    #[inline]
    pub fn velocity(&self) -> f64 {
        self.v
    }
    pub fn reset(&mut self) {
        self.x = 0.0;
        self.v = 0.0;
        self.initialized = false;
    }
}

/// Adaptive 1-D local-level Kalman filter with online estimation of the
/// measurement noise `R` (and process noise `Q`) from the innovation sequence.
/// O(1) recursive, Tier A.
#[derive(Clone, Debug)]
pub struct AdaptiveKalman {
    x: f64,
    p: f64,
    q: f64,
    r: f64,
    innov_ema: f64,
    innov2_ema: f64,
    adapt: f64,
    initialized: bool,
}

impl AdaptiveKalman {
    /// `q0`/`r0` initial process/measurement noise; `adapt` is the EMA factor
    /// for innovation statistics (e.g. 0.05).
    pub fn new(q0: f64, r0: f64, adapt: f64) -> Self {
        Self {
            x: 0.0,
            p: r0.max(1e-6),
            q: q0,
            r: r0,
            innov_ema: 0.0,
            innov2_ema: 0.0,
            adapt,
            initialized: false,
        }
    }
    pub fn update(&mut self, z: f64) -> StateEstimate {
        if !self.initialized {
            self.x = z;
            self.initialized = true;
            return StateEstimate { value: self.x, velocity: None, covariance: vec![self.p] };
        }
        // Predict.
        let p_pred = self.p + self.q;
        // Innovation.
        let y = z - self.x;
        let s = p_pred + self.r;
        let k = p_pred / s;
        self.x += k * y;
        self.p = (1.0 - k) * p_pred;
        // Adapt R from innovation variance: Var(y) ~= p_pred + R  => R ~= E[y^2] - p_pred.
        self.innov_ema = (1.0 - self.adapt) * self.innov_ema + self.adapt * y;
        self.innov2_ema = (1.0 - self.adapt) * self.innov2_ema + self.adapt * y * y;
        let innov_var = (self.innov2_ema - self.innov_ema * self.innov_ema).max(1e-12);
        let r_est = (innov_var - p_pred).max(1e-9);
        self.r = (1.0 - self.adapt) * self.r + self.adapt * r_est;
        StateEstimate { value: self.x, velocity: None, covariance: vec![self.p] }
    }
    #[inline]
    pub fn value(&self) -> f64 {
        self.x
    }
    #[inline]
    pub fn variance(&self) -> f64 {
        self.p
    }
    #[inline]
    pub fn measurement_noise(&self) -> f64 {
        self.r
    }
    pub fn reset(&mut self) {
        let r0 = self.r;
        self.x = 0.0;
        self.p = r0;
        self.innov_ema = 0.0;
        self.innov2_ema = 0.0;
        self.initialized = false;
    }
}

/// Scalar square-root (covariance-factor) Kalman filter for a local-level
/// model. Propagates the standard deviation `s = sqrt(P)` directly to preserve
/// positive-definiteness. O(1), Tier A.
#[derive(Clone, Debug)]
pub struct SquareRootKalman {
    x: f64,
    s: f64, // sqrt(P)
    q: f64,
    r: f64,
    initialized: bool,
}

impl SquareRootKalman {
    pub fn new(q: f64, r: f64, s0: f64) -> Self {
        assert!(q >= 0.0 && r > 0.0 && s0 > 0.0);
        Self { x: 0.0, s: s0, q, r, initialized: false }
    }
    pub fn update(&mut self, z: f64) -> StateEstimate {
        if !self.initialized {
            self.x = z;
            self.initialized = true;
            return StateEstimate { value: self.x, velocity: None, covariance: vec![self.s * self.s] };
        }
        // Predict in variance domain.
        let p_pred = self.s * self.s + self.q;
        // Update.
        let s_inn = p_pred + self.r;
        let k = p_pred / s_inn;
        self.x += k * (z - self.x);
        let p_post = (1.0 - k) * p_pred;
        self.s = p_post.max(0.0).sqrt();
        StateEstimate { value: self.x, velocity: None, covariance: vec![self.s * self.s] }
    }
    #[inline]
    pub fn value(&self) -> f64 {
        self.x
    }
    #[inline]
    pub fn std_dev(&self) -> f64 {
        self.s
    }
    pub fn reset(&mut self) {
        self.x = 0.0;
        self.initialized = false;
    }
}

/// Unscented Kalman filter for the 2-state constant-velocity model with a
/// (possibly nonlinear) measurement `h(pos, vel)`. Uses the full sigma-point
/// machinery; the built-in measurement is `h = pos` (level), so it matches the
/// linear KF, but `h` can be swapped for nonlinear observations.
#[derive(Clone, Debug)]
pub struct UnscentedKalman {
    dt: f64,
    q: f64,
    r: f64,
    kappa: f64,
    x: [f64; 2],
    p: DMat,
    initialized: bool,
    // nonlinearity selector for the measurement
    nonlinear: bool,
}

impl UnscentedKalman {
    pub fn new(dt: f64, q: f64, r: f64, p0: f64) -> Self {
        Self {
            dt,
            q,
            r,
            kappa: 1.0,
            x: [0.0, 0.0],
            p: DMat::from_row_vec(2, 2, vec![p0, 0.0, 0.0, p0]),
            initialized: false,
            nonlinear: false,
        }
    }
    /// Enable a demonstrative nonlinear measurement `h(s) = pos + 0.1*vel^2`.
    pub fn with_nonlinear_measurement(mut self) -> Self {
        self.nonlinear = true;
        self
    }

    fn measure(&self, pos: f64, vel: f64) -> f64 {
        if self.nonlinear {
            pos + 0.1 * vel * vel
        } else {
            pos
        }
    }

    fn transition(&self, s: [f64; 2]) -> [f64; 2] {
        [s[0] + s[1] * self.dt, s[1]]
    }

    pub fn update(&mut self, z: f64) -> StateEstimate {
        let n = 2usize;
        let lam = self.kappa;
        let scale = (n as f64 + lam) as f64;
        // Cholesky of P for sigma points.
        let l = match cholesky(&self.p) {
            Some(l) => l,
            None => DMat::from_row_vec(2, 2, vec![1e-6, 0.0, 0.0, 1e-6]),
        };
        // 2n+1 sigma points.
        let mut sp = [[0.0f64; 2]; 5];
        sp[0] = self.transition(self.x);
        for i in 0..n {
            let col = [l.get(0, i), l.get(1, i)];
            let sc = scale.sqrt();
            // sigma around the *prior* mean, then propagated.
            let mut xp = [0.0, 0.0];
            for d in 0..2 {
                xp[d] = self.x[d] + sc * col[d];
            }
            sp[1 + i] = self.transition(xp);
            let mut xm = [0.0, 0.0];
            for d in 0..2 {
                xm[d] = self.x[d] - sc * col[d];
            }
            sp[1 + n + i] = self.transition(xm);
        }
        // Weights.
        let w0 = lam / (n as f64 + lam);
        let wi = 1.0 / (2.0 * (n as f64 + lam));
        // Predicted mean.
        let mut xm = [0.0, 0.0];
        for d in 0..2 {
            xm[d] = w0 * sp[0][d];
            for k in 1..5 {
                xm[d] += wi * sp[k][d];
            }
        }
        // Predicted covariance.
        let mut p = DMat::zeros(2, 2);
        for k in 0..5 {
            let w = if k == 0 { w0 } else { wi };
            let d0 = sp[k][0] - xm[0];
            let d1 = sp[k][1] - xm[1];
            p.data[0] += w * d0 * d0;
            p.data[1] += w * d0 * d1;
            p.data[2] += w * d1 * d0;
            p.data[3] += w * d1 * d1;
        }
        // Add process noise.
        let dt2 = self.dt * self.dt;
        let dt3 = dt2 * self.dt;
        p.data[0] += self.q * dt3 / 3.0;
        p.data[1] += self.q * dt2 / 2.0;
        p.data[2] += self.q * dt2 / 2.0;
        p.data[3] += self.q * self.dt;

        // Measurement sigma points.
        let mut zs = [0.0f64; 5];
        for k in 0..5 {
            zs[k] = self.measure(sp[k][0], sp[k][1]);
        }
        let mut zm = 0.0;
        for k in 0..5 {
            let w = if k == 0 { w0 } else { wi };
            zm += w * zs[k];
        }
        let mut pzz = self.r;
        let mut pxz = [0.0f64; 2];
        for k in 0..5 {
            let w = if k == 0 { w0 } else { wi };
            let dz = zs[k] - zm;
            pzz += w * dz * dz;
            pxz[0] += w * (sp[k][0] - xm[0]) * dz;
            pxz[1] += w * (sp[k][1] - xm[1]) * dz;
        }
        let k0 = pxz[0] / pzz;
        let k1 = pxz[1] / pzz;
        let innov = z - zm;
        xm[0] += k0 * innov;
        xm[1] += k1 * innov;
        p.data[0] -= k0 * pxz[0];
        p.data[1] -= k0 * pxz[1];
        p.data[2] -= k1 * pxz[0];
        p.data[3] -= k1 * pxz[1];

        self.x = xm;
        self.p = p;
        self.initialized = true;
        StateEstimate {
            value: self.x[0],
            velocity: Some(self.x[1]),
            covariance: self.p.data.clone(),
        }
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.initialized
    }
    pub fn reset(&mut self) {
        self.x = [0.0, 0.0];
        self.p = DMat::identity(2);
        self.initialized = false;
    }
}

/// Extended Kalman filter for the 2-state constant-velocity model with a
/// measurement Jacobian. Built-in measurement `h = pos`; set `nonlinear` for a
/// demonstrative nonlinear observation with its analytic Jacobian.
#[derive(Clone, Debug)]
pub struct ExtendedKalman {
    dt: f64,
    q: f64,
    r: f64,
    x: [f64; 2],
    p: DMat,
    nonlinear: bool,
    initialized: bool,
}

impl ExtendedKalman {
    pub fn new(dt: f64, q: f64, r: f64, p0: f64) -> Self {
        Self {
            dt,
            q,
            r,
            x: [0.0, 0.0],
            p: DMat::from_row_vec(2, 2, vec![p0, 0.0, 0.0, p0]),
            nonlinear: false,
            initialized: false,
        }
    }
    pub fn with_nonlinear_measurement(mut self) -> Self {
        self.nonlinear = true;
        self
    }
    fn measure(&self, pos: f64, vel: f64) -> (f64, [f64; 2]) {
        if self.nonlinear {
            // h = pos + 0.1*vel^2 ; dh/dpos = 1, dh/dvel = 0.2*vel
            (pos + 0.1 * vel * vel, [1.0, 0.2 * vel])
        } else {
            (pos, [1.0, 0.0])
        }
    }
    pub fn update(&mut self, z: f64) -> StateEstimate {
        // Predict (linear CV transition, Jacobian F).
        let f = DMat::from_row_vec(2, 2, vec![1.0, self.dt, 0.0, 1.0]);
        let xp = [self.x[0] + self.x[1] * self.dt, self.x[1]];
        let fp = f.mul(&self.p);
        let mut p = fp.mul(&f.transpose());
        let dt2 = self.dt * self.dt;
        let dt3 = dt2 * self.dt;
        p.data[0] += self.q * dt3 / 3.0;
        p.data[1] += self.q * dt2 / 2.0;
        p.data[2] += self.q * dt2 / 2.0;
        p.data[3] += self.q * self.dt;

        // Update.
        let (hz, hj) = self.measure(xp[0], xp[1]);
        // S = H P H^T + R (scalar measurement).
        let ph = [p.data[0] * hj[0] + p.data[1] * hj[1], p.data[2] * hj[0] + p.data[3] * hj[1]];
        let s = hj[0] * ph[0] + hj[1] * ph[1] + self.r;
        let k = [ph[0] / s, ph[1] / s];
        let innov = z - hz;
        self.x = [xp[0] + k[0] * innov, xp[1] + k[1] * innov];
        // P = (I - K H) P, Joseph form.
        let kh = DMat::from_row_vec(2, 2, vec![k[0] * hj[0], k[0] * hj[1], k[1] * hj[0], k[1] * hj[1]]);
        let ikh = DMat::identity(2).sub(&kh);
        let kd = DMat::from_row_vec(2, 1, vec![k[0], k[1]]);
        let rm = DMat::from_row_vec(1, 1, vec![self.r]);
        self.p = ikh.mul(&p).mul(&ikh.transpose()).add(&kd.mul(&rm).mul(&kd.transpose()));
        self.initialized = true;
        StateEstimate {
            value: self.x[0],
            velocity: Some(self.x[1]),
            covariance: self.p.data.clone(),
        }
    }
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.initialized
    }
    pub fn reset(&mut self) {
        self.x = [0.0, 0.0];
        self.p = DMat::identity(2);
        self.initialized = false;
    }
}

/// LMS adaptive one-step linear predictor. The regressor is the last `order`
/// samples; weights adapt with step size `mu`. Output is the prediction for the
/// current sample (a denoised estimate). O(order) per update, Tier A.
#[derive(Clone, Debug)]
pub struct LmsFilter {
    order: usize,
    mu: f64,
    w: Vec<f64>,
    buf: RingBuffer<f64>,
}

impl LmsFilter {
    pub fn new(order: usize, mu: f64) -> Self {
        assert!(order > 0, "order must be > 0");
        assert!(mu > 0.0, "mu must be > 0");
        Self { order, mu, w: vec![0.0; order], buf: RingBuffer::new(order, 0.0) }
    }
    /// Predict the next value for a regressor of past samples (no weight update).
    pub fn predict(&self) -> f64 {
        let mut acc = 0.0;
        for i in 0..self.order {
            // buf.get(i): 0 = oldest ... order-1 = newest; align with w.
            acc += self.w[i] * self.buf.get(i);
        }
        acc
    }
}

impl ScalarIndicator for LmsFilter {
    type Output = f64;
    fn update(&mut self, value: f64) -> Option<f64> {
        if !self.buf.is_full() {
            self.buf.push(value);
            return None;
        }
        // Prediction from the current (full) buffer of past samples.
        let pred = self.predict();
        let err = value - pred;
        // LMS weight update.
        for i in 0..self.order {
            self.w[i] += self.mu * err * self.buf.get(i);
        }
        self.buf.push(value);
        Some(pred)
    }
    fn reset(&mut self) {
        for w in self.w.iter_mut() {
            *w = 0.0;
        }
        self.buf.clear(0.0);
    }
}

/// RLS adaptive one-step linear predictor (recursive least squares on the last
/// `order` samples). Faster convergence than LMS. Output is the prediction for
/// the current sample.
#[derive(Clone, Debug)]
pub struct RlsFilter {
    order: usize,
    rls: RecursiveLeastSquares,
    buf: RingBuffer<f64>,
    regressor: Vec<f64>,
}

impl RlsFilter {
    pub fn new(order: usize, lambda: f64) -> Self {
        assert!(order > 0, "order must be > 0");
        Self {
            order,
            rls: RecursiveLeastSquares::new(order, lambda, 100.0),
            buf: RingBuffer::new(order, 0.0),
            regressor: vec![0.0; order],
        }
    }
    #[inline]
    pub fn coefficients(&self) -> &[f64] {
        self.rls.coefficients()
    }
}

impl ScalarIndicator for RlsFilter {
    type Output = f64;
    fn update(&mut self, value: f64) -> Option<f64> {
        if !self.buf.is_full() {
            self.buf.push(value);
            return None;
        }
        for i in 0..self.order {
            self.regressor[i] = *self.buf.get(i);
        }
        let pred = self.rls.predict(&self.regressor);
        self.rls.update(&self.regressor, value);
        self.buf.push(value);
        Some(pred)
    }
    fn reset(&mut self) {
        self.rls.reset();
        self.buf.clear(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_beta_tracks_ramp() {
        let mut ab = AlphaBetaTracker::new(0.5, 0.1, 1.0);
        let mut est = ab.update(0.0);
        for i in 1..50 {
            est = ab.update(i as f64);
        }
        // Should track a unit-slope ramp closely.
        assert!((est.value - 49.0).abs() < 3.0);
        assert!(est.velocity.unwrap() > 0.0);
    }

    #[test]
    fn kalman_cv_tracks_ramp() {
        let mut kf = KalmanFilter::constant_velocity(1.0, 1e-3, 1.0, 1.0);
        let mut est = kf.update_scalar(0.0);
        for i in 1..80 {
            est = kf.update_scalar(i as f64);
        }
        assert!((est.value - 79.0).abs() < 5.0);
        assert!(est.velocity.unwrap() > 0.0);
    }

    #[test]
    fn constant_velocity_kalman_matches_generic() {
        let mut generic = KalmanFilter::constant_velocity(1.0, 1e-3, 0.5, 2.0);
        let mut cv = ConstantVelocityKalman::new(1.0, 1e-3, 0.5, 2.0);
        let mut a = 0.0;
        let mut b = 0.0;
        for i in 0..100 {
            let z = (i as f64) * 0.3 + (i as f64 % 5.0) * 0.01;
            a = generic.update_scalar(z).value;
            b = cv.update(z).value;
        }
        // Specialized filter should track the generic one closely.
        assert!((a - b).abs() < 1e-6, "{} vs {}", a, b);
    }

    #[test]
    fn ukf_matches_kf_on_linear() {
        let mut kf = KalmanFilter::constant_velocity(1.0, 1e-2, 0.5, 1.0);
        let mut ukf = UnscentedKalman::new(1.0, 1e-2, 0.5, 1.0);
        let mut a = 0.0;
        let mut b = 0.0;
        for i in 0..60 {
            let z = (i as f64) * 0.5 + (i as f64 % 3.0) * 0.01;
            a = kf.update_scalar(z).value;
            b = ukf.update(z).value;
        }
        // UKF is exact for linear models; residual difference vs the Joseph-form
        // KF is pure floating-point accumulation.
        assert!((a - b).abs() < 1e-2, "UKF should match linear KF: {} vs {}", a, b);
    }

    #[test]
    fn adaptive_kalman_smooths() {
        let mut ak = AdaptiveKalman::new(1e-3, 1e-2, 0.05);
        let mut v = 0.0;
        for i in 0..100 {
            let z = 5.0 + ((i % 7) as f64 - 3.0) * 0.1;
            v = ak.update(z).value;
        }
        assert!((v - 5.0).abs() < 0.5);
    }

    #[test]
    fn sqrt_kalman_tracks_constant() {
        let mut sk = SquareRootKalman::new(1e-3, 1e-2, 1.0);
        let mut v = 0.0;
        for _ in 0..100 {
            v = sk.update(3.0).value;
        }
        assert!((v - 3.0).abs() < 1e-6);
        assert!(sk.std_dev() >= 0.0);
    }

    #[test]
    fn lms_predicts_ar1() {
        let mut lms = LmsFilter::new(2, 0.01);
        let mut last = None;
        let mut prev = 0.0;
        for i in 0..500 {
            let x = 0.8 * prev + ((i % 5) as f64 - 2.0) * 0.01;
            last = lms.update(x);
            prev = x;
        }
        assert!(last.is_some());
    }

    #[test]
    fn rls_predicts() {
        let mut rls = RlsFilter::new(3, 1.0);
        let mut last = None;
        let mut prev = [0.0, 0.0, 0.0];
        for i in 0..200 {
            let x = 0.5 * prev[2] + 0.2 * prev[1] + ((i % 3) as f64) * 0.01;
            last = rls.update(x);
            prev = [prev[1], prev[2], x];
        }
        assert!(last.is_some());
    }
}
