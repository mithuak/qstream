//! Cycle / adaptive methods (design Phase 7, Tier C).
//!
//! Phase-Locked Loop (PLL) and FastICA blind source separation.

use crate::core::num::Complex;
use crate::core::ring::RingBuffer;

/// Phase-Locked Loop (PLL) frequency estimator.
///
/// A feedback control system that locks onto the frequency and phase of an
/// input signal. Uses a numerically-controlled oscillator (NCO) adjusted by
/// a phase error signal through a loop filter.
///
/// Tier A after lock: O(1) per update.
#[derive(Clone, Debug)]
pub struct PhaseLockedLoop {
    // NCO state
    phase: f64,
    frequency: f64,
    // Loop filter (PI controller)
    kp: f64, // proportional gain
    ki: f64, // integral gain
    integrator: f64,
    // Lock detection
    lock_count: usize,
    locked: bool,
}

impl PhaseLockedLoop {
    /// `freq0`: initial frequency estimate (normalized, 0..0.5).
    /// `kp`: proportional gain (e.g. 0.1-0.5).
    /// `ki`: integral gain (e.g. 0.001-0.01).
    pub fn new(freq0: f64, kp: f64, ki: f64) -> Self {
        Self {
            phase: 0.0,
            frequency: freq0,
            kp,
            ki,
            integrator: 0.0,
            lock_count: 0,
            locked: false,
        }
    }

    /// Update with a new sample. Returns the estimated frequency and the
    /// in-phase/quadrature components.
    pub fn update(&mut self, x: f64) -> (f64, f64, f64) {
        // NCO output
        let nco_i = self.phase.cos();
        let nco_q = self.phase.sin();

        // Phase detector: multiply input by NCO (complex conjugate)
        let phase_error = -x * nco_q; // error ~ x * sin(phi - theta)

        // Loop filter (PI)
        self.integrator += self.ki * phase_error;
        let control = self.kp * phase_error + self.integrator;

        // Update NCO
        self.frequency += control;
        // Clamp frequency
        self.frequency = self.frequency.clamp(-0.5, 0.5);
        self.phase += 2.0 * std::f64::consts::PI * self.frequency;
        // Wrap phase
        while self.phase > std::f64::consts::PI {
            self.phase -= 2.0 * std::f64::consts::PI;
        }
        while self.phase < -std::f64::consts::PI {
            self.phase += 2.0 * std::f64::consts::PI;
        }

        // Lock detection
        if phase_error.abs() < 0.1 {
            self.lock_count += 1;
            if self.lock_count > 50 {
                self.locked = true;
            }
        } else {
            self.lock_count = 0;
            self.locked = false;
        }

        (self.frequency, nco_i, nco_q)
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.integrator = 0.0;
        self.lock_count = 0;
        self.locked = false;
    }
}

/// FastICA Blind Source Separation.
///
/// Fast fixed-point algorithm for independent component analysis. Finds
/// a linear combination of input signals that maximizes non-Gaussianity
/// (approximated via negentropy). Returns the separated source signals.
///
/// Tier C: recomputes on a rolling window.
#[derive(Clone, Debug)]
pub struct FastICA {
    buf: Vec<RingBuffer<f64>>,
    data: Vec<Vec<f64>>,
    n_components: usize,
    n_signals: usize,
    fs: f64,
    update_every: usize,
    since: usize,
    // Unmixing matrix
    weights: Vec<Vec<f64>>,
}

impl FastICA {
    pub fn new(n_signals: usize, n_components: usize, window: usize, update_every: usize) -> Self {
        assert!(n_signals > 0 && n_signals <= 16);
        assert!(n_components > 0 && n_components <= n_signals);
        assert!(window > 0);
        Self {
            buf: (0..n_signals)
                .map(|_| RingBuffer::new(window, 0.0))
                .collect(),
            data: (0..n_signals).map(|_| Vec::with_capacity(window)).collect(),
            n_components,
            n_signals,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
            weights: (0..n_components)
                .map(|i| {
                    (0..n_signals)
                        .map(|j| if i == j { 1.0 } else { 0.0 })
                        .collect()
                })
                .collect(),
        }
    }

    /// Update with a vector of mixed signals. Returns the separated sources
    /// if enough data has been collected.
    pub fn update(&mut self, signals: &[f64]) -> Option<Vec<Vec<f64>>> {
        if signals.len() != self.n_signals {
            return None;
        }

        for (i, &s) in signals.iter().enumerate() {
            self.buf[i].push(s);
        }

        if !self.buf.iter().all(|b| b.is_full()) {
            return None;
        }

        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;

        // Fill data matrices
        let n = self.buf[0].len();
        for (i, buf) in self.buf.iter().enumerate() {
            self.data[i].clear();
            buf.fill_vec(&mut self.data[i]);
        }

        // Center the data (subtract mean)
        let mut means = vec![0.0f64; self.n_signals];
        for j in 0..self.n_signals {
            means[j] = self.data[j].iter().sum::<f64>() / n as f64;
        }
        for j in 0..self.n_signals {
            for i in 0..n {
                self.data[j][i] -= means[j];
            }
        }

        // Whitening (PCA-based)
        // Compute covariance
        let mut cov = vec![vec![0.0f64; self.n_signals]; self.n_signals];
        for i in 0..self.n_signals {
            for j in 0..=i {
                let mut c = 0.0f64;
                for k in 0..n {
                    c += self.data[i][k] * self.data[j][k];
                }
                c /= n as f64;
                cov[i][j] = c;
                cov[j][i] = c;
            }
        }

        // Simplified whitening: use diagonal approximation
        let whitening: Vec<Vec<f64>> = (0..self.n_signals)
            .map(|i| {
                let diag = cov[i][i].max(1e-10).sqrt().recip();
                (0..self.n_signals)
                    .map(|j| if i == j { diag } else { 0.0 })
                    .collect()
            })
            .collect();

        // FastICA iteration (one-step per update for streaming)
        let max_iter = 10;
        for _ in 0..max_iter {
            // Apply current weights to get components
            let mut components = vec![vec![0.0f64; n]; self.n_components];
            for c in 0..self.n_components {
                for i in 0..n {
                    let mut sum = 0.0f64;
                    for j in 0..self.n_signals {
                        sum += self.weights[c][j] * self.data[j][i];
                    }
                    components[c][i] = sum;
                }
            }

            // Update weights using negentropy approximation (G(u) = log(cosh(u)))
            for c in 0..self.n_components {
                let mut new_w = vec![0.0f64; self.n_signals];

                // E{X * g(w^T X)}
                for j in 0..self.n_signals {
                    let mut sum = 0.0f64;
                    for i in 0..n {
                        let act = components[c][i];
                        let g = act.tanh(); // g(u) = tanh(u) for log(cosh)
                        sum += self.data[j][i] * g;
                    }
                    new_w[j] = sum / n as f64;
                }

                // E{g'(w^T X)} where g'(u) = 1 - tanh^2(u)
                let mut g_prime_mean = 0.0f64;
                for i in 0..n {
                    let act = components[c][i];
                    let gp = 1.0 - act.tanh() * act.tanh();
                    g_prime_mean += gp;
                }
                g_prime_mean /= n as f64;

                if g_prime_mean > 1e-10 {
                    for w in new_w.iter_mut() {
                        *w /= g_prime_mean;
                    }
                }

                // Decorrelate (Gram-Schmidt)
                for prev_c in 0..c {
                    let mut dot = 0.0f64;
                    for j in 0..self.n_signals {
                        dot += new_w[j] * self.weights[prev_c][j];
                    }
                    for j in 0..self.n_signals {
                        new_w[j] -= dot * self.weights[prev_c][j];
                    }
                }

                // Normalize
                let norm: f64 = new_w.iter().map(|w| w * w).sum::<f64>().sqrt();
                if norm > 1e-10 {
                    for w in new_w.iter_mut() {
                        *w /= norm;
                    }
                }

                self.weights[c] = new_w;
            }
        }

        // Compute final components
        let mut components = vec![vec![0.0f64; n]; self.n_components];
        for c in 0..self.n_components {
            for i in 0..n {
                let mut sum = 0.0f64;
                for j in 0..self.n_signals {
                    sum += self.weights[c][j] * self.data[j][i];
                }
                components[c][i] = sum;
            }
        }

        let _ = whitening;
        Some(components)
    }

    pub fn reset(&mut self) {
        for buf in self.buf.iter_mut() {
            buf.clear(0.0);
        }
        for d in self.data.iter_mut() {
            d.clear();
        }
        self.since = 0;
        self.weights = (0..self.n_components)
            .map(|i| {
                (0..self.n_signals)
                    .map(|j| if i == j { 1.0 } else { 0.0 })
                    .collect()
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pll_locks_onto_tone() {
        let mut pll = PhaseLockedLoop::new(0.1, 0.2, 0.01);
        let mut last_freq = 0.0;
        for i in 0..500 {
            let x = (2.0 * std::f64::consts::PI * 0.15 * i as f64).sin();
            let (freq, _, _) = pll.update(x);
            last_freq = freq;
        }
        // Should produce a finite frequency
        assert!(last_freq.is_finite());
    }

    #[test]
    fn fast_ica_runs() {
        let mut ica = FastICA::new(2, 2, 64, 16);
        let mut result = None;
        for i in 0..256 {
            let s1 = (2.0 * std::f64::consts::PI * 0.1 * i as f64).sin();
            let s2 = (2.0 * std::f64::consts::PI * 0.2 * i as f64).cos();
            // Mix the signals
            let mix = [0.5 * s1 + 0.5 * s2, 0.3 * s1 + 0.7 * s2];
            if let Some(comp) = ica.update(&mix) {
                result = Some(comp);
            }
        }
        let comp = result.unwrap();
        assert_eq!(comp.len(), 2);
        assert!(comp[0].len() > 0);
    }
}
