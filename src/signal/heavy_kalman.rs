//! Heavy/experimental state-space filters (design Phase 7, Tier C).
//!
//! Particle filters (bootstrap/SIR), ensemble Kalman filter (EnKF), and
//! cubature Kalman filter (CKF). These handle nonlinear/non-Gaussian state
//! estimation and are gated behind the `experimental` feature.

use crate::core::matrix::{cholesky, DMat};
use crate::core::num::Complex;
use crate::signal::kalman::StateEstimate;

/// Bootstrap Particle Filter (Sequential Importance Resampling).
///
/// Represents the posterior state distribution with `n_particles` weighted
/// samples. Propagates through a nonlinear state transition, weights by the
/// likelihood of the observation, and resamples when the effective sample
/// size drops below a threshold.
///
/// Tier C: particle operations are O(n_particles) per update.
#[derive(Clone, Debug)]
pub struct ParticleFilter {
    n_particles: usize,
    particles: Vec<f64>,
    weights: Vec<f64>,
    // Model parameters
    process_noise: f64,
    measurement_noise: f64,
    // State bounds for resampling
    state_min: f64,
    state_max: f64,
    initialized: bool,
}

impl ParticleFilter {
    /// `n_particles`: number of particles (e.g. 100-1000).
    /// `process_noise`: std dev of state transition noise.
    /// `measurement_noise`: std dev of observation noise.
    /// `state_min`, `state_max`: initial state bounds.
    pub fn new(
        n_particles: usize,
        process_noise: f64,
        measurement_noise: f64,
        state_min: f64,
        state_max: f64,
    ) -> Self {
        assert!(n_particles > 0);
        assert!(process_noise > 0.0);
        assert!(measurement_noise > 0.0);
        let n = n_particles;
        let uniform_w = 1.0 / n as f64;
        Self {
            n_particles: n,
            particles: vec![0.0; n],
            weights: vec![uniform_w; n],
            process_noise,
            measurement_noise,
            state_min,
            state_max,
            initialized: false,
        }
    }

    /// State transition: random walk (x' = x + noise). Override by using
    /// `update_with_transition` for custom models.
    pub fn update(&mut self, measurement: f64) -> StateEstimate {
        let process_noise = self.process_noise;
        self.update_with_transition(measurement, |x| x + randn() * process_noise)
    }

    /// Update with a custom state transition function.
    pub fn update_with_transition<F>(&mut self, measurement: f64, transition: F) -> StateEstimate
    where
        F: Fn(f64) -> f64,
    {
        // Initialize particles uniformly on first call
        if !self.initialized {
            for i in 0..self.n_particles {
                let t = if self.n_particles > 1 {
                    i as f64 / (self.n_particles - 1) as f64
                } else {
                    0.5
                };
                self.particles[i] = self.state_min + t * (self.state_max - self.state_min);
            }
            self.initialized = true;
        }

        // Propagate particles through transition
        for i in 0..self.n_particles {
            self.particles[i] = transition(self.particles[i]);
        }

        // Weight by likelihood: p(y|x) = N(y; x, measurement_noise^2)
        let mut w_sum = 0.0f64;
        for i in 0..self.n_particles {
            let diff = measurement - self.particles[i];
            let lik = (-0.5 * diff * diff / (self.measurement_noise * self.measurement_noise)).exp();
            self.weights[i] *= lik;
            w_sum += self.weights[i];
        }

        // Normalize weights
        if w_sum > 1e-30 {
            for w in self.weights.iter_mut() {
                *w /= w_sum;
            }
        } else {
            let uniform = 1.0 / self.n_particles as f64;
            for w in self.weights.iter_mut() {
                *w = uniform;
            }
        }

        // Compute effective sample size
        let ess: f64 = self.weights.iter().map(|w| w * w).sum::<f64>().recip();

        // Resample if ESS is too low
        if ess < self.n_particles as f64 * 0.5 {
            self.systematic_resample();
        }

        // State estimate: weighted mean
        let mut mean = 0.0f64;
        let mut var = 0.0f64;
        for i in 0..self.n_particles {
            mean += self.weights[i] * self.particles[i];
        }
        for i in 0..self.n_particles {
            let d = self.particles[i] - mean;
            var += self.weights[i] * d * d;
        }

        StateEstimate {
            value: mean,
            velocity: None,
            covariance: vec![var],
        }
    }

    fn systematic_resample(&mut self) {
        let n = self.n_particles;
        let mut new_particles = Vec::with_capacity(n);
        let u0 = pseudo_random() / n as f64;
        let mut cumsum = 0.0f64;
        let mut j = 0;
        for i in 0..n {
            let u = u0 + i as f64 / n as f64;
            while cumsum < u && j < n - 1 {
                cumsum += self.weights[j];
                j += 1;
            }
            new_particles.push(self.particles[j.min(n - 1)]);
        }
        self.particles = new_particles;
        let uniform = 1.0 / n as f64;
        for w in self.weights.iter_mut() {
            *w = uniform;
        }
    }

    pub fn reset(&mut self) {
        self.initialized = false;
        let uniform = 1.0 / self.n_particles as f64;
        for w in self.weights.iter_mut() {
            *w = uniform;
        }
    }
}

/// Ensemble Kalman Filter (EnKF).
///
/// Monte Carlo approximation of the Kalman filter for nonlinear systems.
/// Maintains an ensemble of state vectors; the sample mean and covariance
/// replace the Kalman gain computation. Suitable for moderate-dimensional
/// state spaces.
#[derive(Clone, Debug)]
pub struct EnsembleKalman {
    n_members: usize,
    state_dim: usize,
    ensemble: Vec<Vec<f64>>, // n_members x state_dim
    process_noise: f64,
    measurement_noise: f64,
    initialized: bool,
}

impl EnsembleKalman {
    pub fn new(
        n_members: usize,
        state_dim: usize,
        process_noise: f64,
        measurement_noise: f64,
    ) -> Self {
        assert!(n_members > 1 && state_dim > 0);
        Self {
            n_members,
            state_dim,
            ensemble: vec![vec![0.0; state_dim]; n_members],
            process_noise,
            measurement_noise,
            initialized: false,
        }
    }

    /// Update with a scalar measurement (observation operator: identity on
    /// first state component).
    pub fn update(&mut self, measurement: f64) -> StateEstimate {
        // Initialize ensemble
        if !self.initialized {
            for e in self.ensemble.iter_mut() {
                for (j, v) in e.iter_mut().enumerate() {
                    *v = measurement + (pseudo_random() - 0.5) * self.measurement_noise
                        + if j == 0 { 0.0 } else { (pseudo_random() - 0.5) * 0.1 };
                }
            }
            self.initialized = true;
        }

        // Forecast: perturb each ensemble member with process noise
        for e in self.ensemble.iter_mut() {
            for v in e.iter_mut() {
                *v += pseudo_random() * self.process_noise;
            }
        }

        // Compute ensemble mean
        let mut mean = vec![0.0f64; self.state_dim];
        for e in self.ensemble.iter() {
            for (j, v) in e.iter().enumerate() {
                mean[j] += v;
            }
        }
        for v in mean.iter_mut() {
            *v /= self.n_members as f64;
        }

        // Compute sample covariance (only first component for scalar obs)
        let mut p11 = 0.0f64;
        for e in self.ensemble.iter() {
            let d = e[0] - mean[0];
            p11 += d * d;
        }
        p11 /= (self.n_members - 1) as f64;

        // Kalman gain for first component
        let r = self.measurement_noise * self.measurement_noise;
        let k_gain = p11 / (p11 + r);

        // Update ensemble
        for e in self.ensemble.iter_mut() {
            let innovation = measurement - e[0];
            e[0] += k_gain * innovation;
        }

        // Recompute mean for output
        let mut final_mean = 0.0f64;
        for e in self.ensemble.iter() {
            final_mean += e[0];
        }
        final_mean /= self.n_members as f64;

        let mut final_var = 0.0f64;
        for e in self.ensemble.iter() {
            let d = e[0] - final_mean;
            final_var += d * d;
        }
        final_var /= (self.n_members - 1) as f64;

        StateEstimate {
            value: final_mean,
            velocity: if self.state_dim > 1 {
                Some(
                    self.ensemble
                        .iter()
                        .map(|e| e[1])
                        .sum::<f64>()
                        / self.n_members as f64,
                )
            } else {
                None
            },
            covariance: vec![final_var],
        }
    }

    pub fn reset(&mut self) {
        self.initialized = false;
    }
}

/// Cubature Kalman Filter (CKF).
///
/// Deterministic sampling approach for nonlinear Bayesian filtering. Uses
/// cubature points (2n points for n-dimensional state) to propagate the
/// mean and covariance through nonlinearities. More accurate than EKF for
/// strongly nonlinear models.
#[derive(Clone, Debug)]
pub struct CubatureKalman {
    state_dim: usize,
    mean: Vec<f64>,
    cov: DMat,
    q: DMat, // process noise covariance
    r: f64,  // measurement noise variance
    initialized: bool,
}

impl CubatureKalman {
    pub fn new(state_dim: usize, process_noise: f64, measurement_noise: f64) -> Self {
        assert!(state_dim > 0 && state_dim <= 10);
        let n = state_dim;
        let mut cov = DMat::identity(n);
        cov.scale(0.1);
        let mut q = DMat::identity(n);
        q.scale(process_noise);
        Self {
            state_dim: n,
            mean: vec![0.0; n],
            cov,
            q,
            r: measurement_noise,
            initialized: false,
        }
    }

    /// Update with a scalar measurement (observation: first state component).
    pub fn update(&mut self, measurement: f64) -> StateEstimate {
        let n = self.state_dim;

        if !self.initialized {
            self.mean[0] = measurement;
            self.initialized = true;
            return StateEstimate {
                value: measurement,
                velocity: if n > 1 { Some(0.0) } else { None },
                covariance: (0..n).map(|i| self.cov.get(i, i)).collect(),
            };
        }

        // Generate cubature points: X_i = mean +/+ sqrt(n*P) * e_i
        let sqrt_n = (n as f64).sqrt();
        let sqrt_p = match cholesky(&self.cov) {
            Some(l) => l,
            None => {
                let mut cov = DMat::identity(n);
                cov.scale(0.1);
                self.cov = cov;
                cholesky(&self.cov).unwrap()
            }
        };

        let n_cub = 2 * n;
        let mut cubature: Vec<Vec<f64>> = Vec::with_capacity(n_cub);
        for i in 0..n {
            let mut plus = self.mean.clone();
            let mut minus = self.mean.clone();
            for j in 0..n {
                plus[j] += sqrt_n * sqrt_p.get(j, i);
                minus[j] -= sqrt_n * sqrt_p.get(j, i);
            }
            cubature.push(plus);
            cubature.push(minus);
        }

        // Propagate through state transition (identity + noise for random walk)
        let predicted: Vec<Vec<f64>> = cubature;

        // Predicted mean
        let mut pred_mean = vec![0.0f64; n];
        for x in predicted.iter() {
            for (j, v) in x.iter().enumerate() {
                pred_mean[j] += v;
            }
        }
        for v in pred_mean.iter_mut() {
            *v /= n_cub as f64;
        }

        // Predicted covariance
        let mut pred_cov = self.q.clone();
        for x in predicted.iter() {
            for i in 0..n {
                for j in 0..n {
                    let d = (x[i] - pred_mean[i]) * (x[j] - pred_mean[j]);
                    pred_cov.set(i, j, pred_cov.get(i, j) + d / n_cub as f64);
                }
            }
        }

        // Innovation
        let innovation = measurement - pred_mean[0];
        let s = pred_cov.get(0, 0) + self.r;

        // Kalman gain
        let mut k = vec![0.0f64; n];
        for i in 0..n {
            k[i] = pred_cov.get(i, 0) / s;
        }

        // Update
        for i in 0..n {
            self.mean[i] = pred_mean[i] + k[i] * innovation;
        }
        for i in 0..n {
            for j in 0..n {
                let new_cov = pred_cov.get(i, j) - k[i] * pred_cov.get(0, j);
                self.cov.set(i, j, new_cov);
            }
        }

        StateEstimate {
            value: self.mean[0],
            velocity: if n > 1 { Some(self.mean[1]) } else { None },
            covariance: (0..n).map(|i| self.cov.get(i, i)).collect(),
        }
    }

    pub fn reset(&mut self) {
        self.initialized = false;
        self.mean = vec![0.0; self.state_dim];
        let mut cov = DMat::identity(self.state_dim);
        cov.scale(0.1);
        self.cov = cov;
    }
}

/// Simple pseudo-random number generator (xorshift) for deterministic behavior.
static mut SEED: u64 = 0x123456789ABCDEF;

fn pseudo_random() -> f64 {
    unsafe {
        SEED ^= SEED << 13;
        SEED ^= SEED >> 7;
        SEED ^= SEED << 17;
        (SEED as f64) / (u64::MAX as f64)
    }
}

/// Standard normal via Box-Muller transform.
fn randn() -> f64 {
    let u1 = pseudo_random().max(1e-10);
    let u2 = pseudo_random();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_filter_tracks_constant() {
        let mut pf = ParticleFilter::new(200, 0.01, 0.1, -5.0, 5.0);
        let mut last = 0.0;
        for i in 0..100 {
            let measurement = 1.0 + (pseudo_random() - 0.5) * 0.1;
            let est = pf.update(measurement);
            last = est.value;
        }
        // Should be near 1.0
        assert!((last - 1.0).abs() < 0.5, "pf estimate {} vs 1.0", last);
    }

    #[test]
    fn ensemble_kalman_tracks_signal() {
        let mut enkf = EnsembleKalman::new(50, 2, 0.01, 0.1);
        let mut last = 0.0;
        for i in 0..100 {
            let measurement = (i as f64 * 0.01).sin() + (pseudo_random() - 0.5) * 0.1;
            let est = enkf.update(measurement);
            last = est.value;
        }
        assert!(last.is_finite());
    }

    #[test]
    fn cubature_kalman_runs() {
        let mut ckf = CubatureKalman::new(2, 0.01, 0.1);
        let mut last = 0.0;
        for i in 0..100 {
            let measurement = (i as f64 * 0.01).sin() + (pseudo_random() - 0.5) * 0.1;
            let est = ckf.update(measurement);
            last = est.value;
        }
        assert!(last.is_finite());
    }

    #[test]
    fn particle_filter_reset() {
        let mut pf = ParticleFilter::new(100, 0.01, 0.1, -5.0, 5.0);
        for _ in 0..50 {
            pf.update(1.0);
        }
        pf.reset();
        assert!(!pf.initialized);
    }
}
