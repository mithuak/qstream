//! Heavy/experimental signal decomposition methods (design Phase 7, Tier C).
//!
//! Non-recursive decomposition algorithms: VMD (Variational Mode
//! Decomposition), EMD (Empirical Mode Decomposition / Hilbert-Huang),
//! LMD (Local Mean Decomposition), Matching Pursuit, and Synchrosqueezing.
//! These push every observation into a ring buffer but only recompute on an
//! `update_every` cadence.

use crate::core::fft::{next_pow2, FftPlan};
use crate::core::num::Complex;
use crate::core::ring::RingBuffer;
use crate::signal::spectral::SpectrumResult;

/// Result from a decomposition method: a set of mode/intrinsic-mode-function
/// coefficients and their per-mode dominant frequencies.
#[derive(Clone, Debug)]
pub struct DecompositionResult {
    /// Per-mode coefficients (flattened: mode0_t0, mode0_t1, ..., mode1_t0, ...).
    pub coefficients: Vec<f64>,
    /// Number of modes.
    pub n_modes: usize,
    /// Per-mode dominant frequency estimate.
    pub mode_frequencies: Vec<f64>,
}

/// Variational Mode Decomposition (VMD).
///
/// Non-recursively decomposes a signal into `n_modes` band-limited intrinsic
/// mode functions (IMFs) by solving a variational problem in the frequency
/// domain via alternating direction method of multipliers (ADMM). Each mode
/// is compact around a center frequency learned during optimization.
///
/// Tier C: recomputes only every `update_every` samples.
#[derive(Clone, Debug)]
pub struct VmdDecomposition {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    n_modes: usize,
    alpha: f64,
    tau: f64,
    fs: f64,
    update_every: usize,
    since: usize,
    // Scratch
    modes: Vec<Vec<f64>>,
    center_freqs: Vec<f64>,
    plan: FftPlan,
    fft_scratch: Vec<Complex>,
}

impl VmdDecomposition {
    /// `window`: analysis window length.
    /// `n_modes`: number of modes K to extract.
    /// `alpha`: bandwidth constraint (typically 1000-3000).
    /// `tau`: dual ascent step size (0 for no dual).
    /// `update_every`: recompute cadence.
    pub fn new(window: usize, n_modes: usize, alpha: f64, update_every: usize) -> Self {
        assert!(window > 0 && n_modes > 0 && n_modes <= 16);
        let nfft = next_pow2(window);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            n_modes,
            alpha,
            tau: 0.0,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
            modes: vec![vec![0.0; window]; n_modes],
            center_freqs: (0..n_modes)
                .map(|k| (k as f64 + 0.5) / n_modes as f64 * 0.5)
                .collect(),
            plan: FftPlan::new(nfft),
            fft_scratch: Vec::with_capacity(nfft),
        }
    }

    pub fn update(&mut self, x: f64) -> Option<DecompositionResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();
        let k = self.n_modes;
        let nfft = next_pow2(n);

        // Initialize modes as uniform band-limited signals
        for m in 0..k {
            self.modes[m].resize(n, 0.0);
        }

        // FFT of the input signal
        self.fft_scratch.clear();
        for i in 0..n {
            self.fft_scratch.push(Complex::new(self.data[i], 0.0));
        }
        self.fft_scratch.resize(nfft, Complex::new(0.0, 0.0));
        self.plan.fft(&mut self.fft_scratch);
        let u_hat: Vec<Complex> = self.fft_scratch.clone();

        // ADMM iterations
        let max_iter = 50;
        let tol = 1e-6;

        // Mode spectra in frequency domain
        let mut v_hat: Vec<Vec<Complex>> = vec![vec![Complex::new(0.0, 0.0); nfft]; k];
        let mut lambda_hat = vec![Complex::new(0.0, 0.0); nfft];
        let mut omega: Vec<f64> = self.center_freqs.clone();

        for _iter in 0..max_iter {
            for m in 0..k {
                // Update mode spectrum: v_hat_m = (u_hat - sum_{i!=m} v_i + lambda/2) / (1 + 2 alpha (w - omega_m)^2)
                for f in 0..nfft {
                    let w = freq_to_norm(f, nfft);
                    let mut sum_others = Complex::new(0.0, 0.0);
                    for i in 0..k {
                        if i != m {
                            sum_others = sum_others + v_hat[i][f];
                        }
                    }
                    let numerator = u_hat[f] - sum_others + lambda_hat[f] * Complex::new(0.5, 0.0);
                    let denom = 1.0 + 2.0 * self.alpha * (w - omega[m]).powi(2);
                    v_hat[m][f] = numerator * Complex::new(1.0 / denom, 0.0);
                }
            }

            // Update center frequencies
            for m in 0..k {
                let mut num = 0.0f64;
                let mut den = 0.0f64;
                for f in 0..nfft / 2 {
                    let w = f as f64 / nfft as f64;
                    let p = v_hat[m][f].norm_sqr();
                    num += w * p;
                    den += p;
                }
                if den > 1e-30 {
                    omega[m] = num / den;
                }
            }

            // Update dual: lambda = lambda + tau * (u - sum v)
            let mut max_diff = 0.0f64;
            for f in 0..nfft {
                let mut sum_v = Complex::new(0.0, 0.0);
                for m in 0..k {
                    sum_v = sum_v + v_hat[m][f];
                }
                let diff = u_hat[f] - sum_v;
                if self.tau > 0.0 {
                    lambda_hat[f] = lambda_hat[f] + Complex::new(self.tau, 0.0) * diff;
                }
                max_diff = max_diff.max(diff.norm_sqr());
            }

            if max_diff < tol * tol {
                break;
            }
        }

        // Inverse FFT each mode to get time-domain IMFs
        let mut coefficients = Vec::with_capacity(k * n);
        let mut mode_frequencies = Vec::with_capacity(k);

        for m in 0..k {
            self.fft_scratch = v_hat[m].clone();
            self.plan.ifft(&mut self.fft_scratch);
            for i in 0..n {
                coefficients.push(self.fft_scratch[i].re / nfft as f64);
            }
            mode_frequencies.push(omega[m] * self.fs);
        }

        Some(DecompositionResult {
            coefficients,
            n_modes: k,
            mode_frequencies,
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Convert FFT bin index to normalized frequency in [0, 0.5].
fn freq_to_norm(bin: usize, nfft: usize) -> f64 {
    if bin <= nfft / 2 {
        bin as f64 / nfft as f64
    } else {
        (bin as f64 - nfft as f64) / nfft as f64
    }
}

/// Empirical Mode Decomposition (EMD) / Hilbert-Huang.
///
/// Sifts the signal to extract intrinsic mode functions (IMFs) adaptively.
/// The number of IMFs is determined by the sifting process. Returns the
/// extracted IMFs and their instantaneous frequencies estimated via the
/// Hilbert transform at the window center.
#[derive(Clone, Debug)]
pub struct EmdDecomposition {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    max_imfs: usize,
    fs: f64,
    update_every: usize,
    since: usize,
}

impl EmdDecomposition {
    pub fn new(window: usize, max_imfs: usize, update_every: usize) -> Self {
        assert!(window > 0 && max_imfs > 0 && max_imfs <= 16);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            max_imfs,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<DecompositionResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();
        let mut residual = self.data.clone();
        let mut imfs: Vec<Vec<f64>> = Vec::new();
        let mut mode_frequencies: Vec<f64> = Vec::new();

        for _imf_idx in 0..self.max_imfs {
            let mut h = residual.clone();

            // Sifting
            for _sift in 0..10 {
                let (max_env, min_env) = envelope(&h);
                if max_env.len() != h.len() {
                    break;
                }
                let mean_env: Vec<f64> = max_env
                    .iter()
                    .zip(min_env.iter())
                    .map(|(a, b)| (a + b) * 0.5)
                    .collect();

                let h_new: Vec<f64> = h.iter().zip(mean_env.iter()).map(|(hi, mi)| hi - mi).collect();

                // Check IMF condition: number of extrema and zero-crossings differ by at most 1
                let n_extrema = count_extrema(&h_new);
                let n_zc = count_zero_crossings(&h_new);
                if (n_extrema as i64 - n_zc as i64).abs() <= 1 {
                    h = h_new;
                    break;
                }
                h = h_new;
            }

            // Check if residual is monotonic
            if count_extrema(&residual) <= 2 {
                break;
            }

            // Estimate dominant frequency of this IMF via zero-crossing rate
            let zc = count_zero_crossings(&h);
            let freq = zc as f64 / (2.0 * n as f64) * self.fs;
            mode_frequencies.push(freq);

            // Subtract IMF from residual
            for i in 0..n {
                residual[i] -= h[i];
            }
            imfs.push(h);

            if count_extrema(&residual) <= 2 {
                break;
            }
        }

        // Flatten IMFs into coefficients
        let mut coefficients = Vec::new();
        for imf in &imfs {
            coefficients.extend_from_slice(imf);
        }

        Some(DecompositionResult {
            coefficients,
            n_modes: imfs.len(),
            mode_frequencies,
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Compute upper and lower envelopes via cubic spline interpolation
/// (simplified: linear interpolation between extrema).
fn envelope(signal: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = signal.len();
    if n < 3 {
        return (vec![0.0; n], vec![0.0; n]);
    }

    // Find local maxima and minima indices
    let mut max_idx: Vec<usize> = Vec::new();
    let mut min_idx: Vec<usize> = Vec::new();

    for i in 1..n - 1 {
        if signal[i] >= signal[i - 1] && signal[i] >= signal[i + 1] {
            max_idx.push(i);
        }
        if signal[i] <= signal[i - 1] && signal[i] <= signal[i + 1] {
            min_idx.push(i);
        }
    }

    if max_idx.len() < 2 || min_idx.len() < 2 {
        return (vec![0.0; n], vec![0.0; n]);
    }

    let max_env = interpolate_envelope(signal, &max_idx);
    let min_env = interpolate_envelope(signal, &min_idx);
    (max_env, min_env)
}

fn interpolate_envelope(signal: &[f64], extrema_idx: &[usize]) -> Vec<f64> {
    let n = signal.len();
    let mut env = vec![0.0; n];

    if extrema_idx.is_empty() {
        return env;
    }

    // Extend endpoints
    let first_idx = extrema_idx[0];
    let last_idx = extrema_idx[extrema_idx.len() - 1];

    for i in 0..n {
        if i <= first_idx {
            env[i] = signal[first_idx];
        } else if i >= last_idx {
            env[i] = signal[last_idx];
        } else {
            // Find surrounding extrema
            let mut j = 0;
            while j < extrema_idx.len() - 1 && extrema_idx[j + 1] < i {
                j += 1;
            }
            if j + 1 < extrema_idx.len() {
                let i0 = extrema_idx[j];
                let i1 = extrema_idx[j + 1];
                let t = (i - i0) as f64 / (i1 - i0).max(1) as f64;
                env[i] = signal[i0] * (1.0 - t) + signal[i1] * t;
            }
        }
    }
    env
}

fn count_extrema(signal: &[f64]) -> usize {
    let mut count = 0;
    for i in 1..signal.len() - 1 {
        if (signal[i] > signal[i - 1] && signal[i] > signal[i + 1])
            || (signal[i] < signal[i - 1] && signal[i] < signal[i + 1])
        {
            count += 1;
        }
    }
    count
}

fn count_zero_crossings(signal: &[f64]) -> usize {
    let mut count = 0;
    for i in 1..signal.len() {
        if (signal[i - 1] >= 0.0 && signal[i] < 0.0) || (signal[i - 1] < 0.0 && signal[i] >= 0.0) {
            count += 1;
        }
    }
    count
}

/// Local Mean Decomposition (LMD).
///
/// Decomposes a signal into a set of product functions (PFs), each being the
/// product of an envelope signal and a purely frequency-modulated signal.
/// The envelope is estimated via local means between extrema.
#[derive(Clone, Debug)]
pub struct LmdDecomposition {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    max_pfs: usize,
    fs: f64,
    update_every: usize,
    since: usize,
}

impl LmdDecomposition {
    pub fn new(window: usize, max_pfs: usize, update_every: usize) -> Self {
        assert!(window > 0 && max_pfs > 0 && max_pfs <= 16);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            max_pfs,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<DecompositionResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();
        let mut residual = self.data.clone();
        let mut pfs: Vec<Vec<f64>> = Vec::new();
        let mut mode_frequencies: Vec<f64> = Vec::new();

        for _pf_idx in 0..self.max_pfs {
            let mut h = residual.clone();
            let mut prev_mean: Option<Vec<f64>> = None;

            for _iter in 0..20 {
                let (max_env, min_env) = envelope(&h);
                if max_env.len() != h.len() {
                    break;
                }
                let local_mean: Vec<f64> = max_env
                    .iter()
                    .zip(min_env.iter())
                    .map(|(a, b)| (a + b) * 0.5)
                    .collect();

                let h_new: Vec<f64> = h.iter().zip(local_mean.iter()).map(|(hi, mi)| hi - mi).collect();

                // Envelope estimate
                let env_est = {
                    let (me, _) = envelope(&h_new);
                    me
                };

                // Normalize: f = h_new / envelope
                let f_new: Vec<f64> = h_new
                    .iter()
                    .zip(env_est.iter())
                    .map(|(hi, ei)| if *ei > 1e-10 { hi / ei } else { *hi })
                    .collect();

                // Check convergence
                if let Some(ref pm) = prev_mean {
                    let diff: f64 = local_mean
                        .iter()
                        .zip(pm.iter())
                        .map(|(a, b)| (a - b).abs())
                        .sum::<f64>()
                        / n as f64;
                    if diff < 1e-4 {
                        h = f_new;
                        break;
                    }
                }
                prev_mean = Some(local_mean);
                h = f_new;
            }

            // Estimate frequency via zero-crossings
            let zc = count_zero_crossings(&h);
            let freq = zc as f64 / (2.0 * n as f64) * self.fs;
            mode_frequencies.push(freq);

            for i in 0..n {
                residual[i] -= h[i];
            }
            pfs.push(h);

            if count_extrema(&residual) <= 2 {
                break;
            }
        }

        let mut coefficients = Vec::new();
        for pf in &pfs {
            coefficients.extend_from_slice(pf);
        }

        Some(DecompositionResult {
            coefficients,
            n_modes: pfs.len(),
            mode_frequencies,
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Matching Pursuit greedy sparse decomposition.
///
/// Iteratively selects the atom from a dictionary (Gabor atoms: Gaussian
/// windowed sinusoids) that best correlates with the signal, subtracts its
/// contribution, and repeats for `n_atoms` iterations. Returns the
/// reconstructed components and their frequencies.
#[derive(Clone, Debug)]
pub struct MatchingPursuitDecomposition {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    n_atoms: usize,
    fs: f64,
    update_every: usize,
    since: usize,
}

impl MatchingPursuitDecomposition {
    pub fn new(window: usize, n_atoms: usize, update_every: usize) -> Self {
        assert!(window > 0 && n_atoms > 0 && n_atoms <= 32);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            n_atoms,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<DecompositionResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();
        let mut residual = self.data.clone();
        let mut components: Vec<Vec<f64>> = Vec::new();
        let mut mode_frequencies: Vec<f64> = Vec::new();

        // Gabor atom dictionary: frequencies x scales
        let freqs: Vec<f64> = (1..=16).map(|k| k as f64 / n as f64).collect();
        let scales: Vec<f64> = vec![4.0, 8.0, 16.0, 32.0];

        for _atom in 0..self.n_atoms {
            let mut best_corr = 0.0f64;
            let mut best_atom: Vec<f64> = vec![0.0; n];
            let mut best_freq = 0.0f64;

            for &f0 in &freqs {
                for &sigma in &scales {
                    let atom = gabor_atom(n, f0, sigma);
                    let corr: f64 = residual
                        .iter()
                        .zip(atom.iter())
                        .map(|(r, a)| r * a)
                        .sum();
                    if corr.abs() > best_corr.abs() {
                        best_corr = corr.abs();
                        best_atom = atom;
                        best_freq = f0;
                    }
                }
            }

            if best_corr < 1e-10 {
                break;
            }

            // Subtract projection
            let projection: Vec<f64> = best_atom.iter().map(|a| a * best_corr).collect();
            for i in 0..n {
                residual[i] -= projection[i];
            }
            components.push(projection);
            mode_frequencies.push(best_freq * self.fs);
        }

        let mut coefficients = Vec::new();
        for comp in &components {
            coefficients.extend_from_slice(comp);
        }

        Some(DecompositionResult {
            coefficients,
            n_modes: components.len(),
            mode_frequencies,
        })
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Generate a Gabor atom: Gaussian-windowed complex sinusoid (real part).
fn gabor_atom(n: usize, freq: f64, sigma: f64) -> Vec<f64> {
    let mut atom = vec![0.0; n];
    let center = n as f64 / 2.0;
    let norm = (std::f64::consts::PI * sigma * sigma).sqrt() * n as f64;
    for i in 0..n {
        let t = (i as f64 - center) / sigma;
        let envelope = (-0.5 * t * t).exp();
        atom[i] = envelope * (2.0 * std::f64::consts::PI * freq * i as f64).cos() / norm.max(1e-10);
    }
    // Normalize
    let energy: f64 = atom.iter().map(|a| a * a).sum::<f64>().sqrt();
    if energy > 1e-10 {
        for a in atom.iter_mut() {
            *a /= energy;
        }
    }
    atom
}

/// Synchrosqueezed STFT.
///
/// Sharpens a standard STFT by reassigning energy to the instantaneous
/// frequency estimate at each time-frequency bin. Produces a concentrated
/// time-frequency representation. Returns the sharpened spectrum at the
/// window center.
#[derive(Clone, Debug)]
pub struct SynchrosqueezingTransform {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    nfft: usize,
    fs: f64,
    update_every: usize,
    since: usize,
    plan: FftPlan,
    win: Vec<f64>,
}

impl SynchrosqueezingTransform {
    pub fn new(window: usize, nfft: usize, update_every: usize) -> Self {
        assert!(window > 0 && nfft > 0);
        let nfft = next_pow2(nfft);
        let win = hann_window(window);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            nfft,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
            plan: FftPlan::new(nfft),
            win,
        }
    }

    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);

        let n = self.data.len();
        let nfft = self.nfft;

        // Compute STFT with a small hop to get two consecutive frames for
        // instantaneous frequency estimation.
        let half = n / 2;
        let mut frame1 = vec![Complex::new(0.0, 0.0); nfft];
        let mut frame2 = vec![Complex::new(0.0, 0.0); nfft];

        for i in 0..n.min(half) {
            frame1[i] = Complex::new(self.data[i] * self.win[i], 0.0);
        }
        for i in 0..n.min(half) {
            frame2[i] = Complex::new(self.data[i + half] * self.win[i], 0.0);
        }

        self.plan.fft(&mut frame1);
        self.plan.fft(&mut frame2);

        // Synchrosqueezing: reassign each bin to the instantaneous frequency.
        let bins = nfft / 2 + 1;
        let mut squeezed = vec![0.0f64; bins];

        for k in 0..bins {
            // Instantaneous frequency estimate from phase difference
            let phase1 = frame1[k].arg();
            let phase2 = frame2[k].arg();
            let mut dp = phase2 - phase1;
            // Wrap to [-pi, pi]
            while dp > std::f64::consts::PI {
                dp -= 2.0 * std::f64::consts::PI;
            }
            while dp < -std::f64::consts::PI {
                dp += 2.0 * std::f64::consts::PI;
            }
            let if_est = k as f64 / nfft as f64 + dp / (2.0 * std::f64::consts::PI * half as f64);

            // Reassign to nearest bin
            let target_bin = (if_est * nfft as f64).round() as isize;
            if target_bin >= 0 && target_bin < bins as isize {
                let tb = target_bin as usize;
                let power = frame1[k].norm_sqr();
                squeezed[tb] += power;
            } else {
                squeezed[k] += frame1[k].norm_sqr();
            }
        }

        let freqs: Vec<f64> = (0..bins).map(|k| k as f64 * self.fs / nfft as f64).collect();
        Some(SpectrumResult::from_psd(freqs, squeezed))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

fn hann_window(n: usize) -> Vec<f64> {
    if n <= 1 {
        return vec![1.0; n];
    }
    (0..n)
        .map(|i| {
            0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / (n - 1) as f64).cos()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(n: usize, freq: f64) -> Vec<f64> {
        (0..n)
            .map(|i| (2.0 * std::f64::consts::PI * freq * i as f64).sin())
            .collect()
    }

    #[test]
    fn vmd_runs() {
        let mut vmd = VmdDecomposition::new(64, 3, 2000.0, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = vmd.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.n_modes > 0);
        assert_eq!(r.mode_frequencies.len(), r.n_modes);
    }

    #[test]
    fn emd_runs() {
        let mut emd = EmdDecomposition::new(64, 4, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = emd.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.n_modes > 0);
    }

    #[test]
    fn lmd_runs() {
        let mut lmd = LmdDecomposition::new(64, 4, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = lmd.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.n_modes > 0);
    }

    #[test]
    fn matching_pursuit_runs() {
        let mut mp = MatchingPursuitDecomposition::new(64, 5, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = mp.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.n_modes > 0);
    }

    #[test]
    fn synchrosqueezing_runs() {
        let mut sq = SynchrosqueezingTransform::new(64, 128, 16);
        let sig = sine(256, 0.15);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = sq.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!(r.power.iter().all(|p| p.is_finite()));
    }
}
