//! Time-frequency features: Short-Time Fourier Transform (spectrogram) and the
//! Hilbert transform (analytic signal -> instantaneous amplitude, phase, and
//! frequency). Both are window/FFT based (Tier C) and recompute on a cadence.

use crate::core::fft::{next_pow2, FftPlan};
use crate::core::num::Complex;
use crate::core::ring::RingBuffer;
use crate::core::window::{fill_window, WindowKind};
use crate::signal::spectral::SpectrumResult;

const TAU: f64 = 2.0 * std::f64::consts::PI;

/// Short-Time Fourier Transform producing a running spectrogram. Each hop
/// returns the magnitude spectrum of the current window as a [`SpectrumResult`]
/// and appends it to an internal spectrogram (bounded to `max_frames`).
#[derive(Clone, Debug)]
pub struct ShortTimeFourierTransform {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    plan: FftPlan,
    win: Vec<f64>,
    scratch: Vec<Complex>,
    fs: f64,
    hop: usize,
    since: usize,
    bins: usize,
    window: usize,
    /// Flattened spectrogram frames (row-major: frame * bins + k), most recent
    /// frames kept up to `max_frames`.
    frames: Vec<f64>,
    max_frames: usize,
}

impl ShortTimeFourierTransform {
    pub fn new(window: usize, hop: usize, fs: f64, kind: WindowKind, max_frames: usize) -> Self {
        let w = next_pow2(window);
        let mut win = Vec::new();
        fill_window(kind, w, &mut win);
        let bins = w / 2 + 1;
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            plan: FftPlan::new(w),
            win,
            scratch: Vec::with_capacity(w),
            fs,
            hop: hop.max(1),
            since: 0,
            bins,
            window: w,
            frames: Vec::new(),
            max_frames: max_frames.max(1),
        }
    }

    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.hop {
            return None;
        }
        self.since = 0;
        self.buf.fill_vec(&mut self.data);
        self.scratch.clear();
        for i in 0..self.window {
            self.scratch.push(Complex::new(self.data[i] * self.win[i], 0.0));
        }
        self.plan.fft(&mut self.scratch);
        let mut freqs = Vec::with_capacity(self.bins);
        let mut power = Vec::with_capacity(self.bins);
        for k in 0..self.bins {
            freqs.push(k as f64 * self.fs / self.window as f64);
            power.push(self.scratch[k].abs());
        }
        // Append frame to the bounded spectrogram.
        self.frames.extend_from_slice(&power);
        while self.frames.len() > self.max_frames * self.bins {
            self.frames.drain(0..self.bins);
        }
        Some(SpectrumResult::from_psd(freqs, power))
    }

    /// Spectrogram as `(n_frames, n_bins, flat_magnitudes)` row-major.
    pub fn spectrogram(&self) -> (usize, usize, Vec<f64>) {
        (self.frames.len() / self.bins, self.bins, self.frames.clone())
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.scratch.clear();
        self.frames.clear();
        self.since = 0;
    }
}

/// Hilbert transform via the FFT analytic-signal construction. On each cadence
/// it returns the instantaneous amplitude (envelope), phase, and frequency at
/// the most recent sample.
#[derive(Clone, Debug)]
pub struct HilbertTransform {
    buf: RingBuffer<f64>,
    plan: FftPlan,
    scratch: Vec<Complex>,
    fs: f64,
    update_every: usize,
    since: usize,
    window: usize,
    prev_phase: Option<f64>,
}

/// Wrap an angle difference into `[-pi, pi]`.
fn wrap_pi(a: f64) -> f64 {
    let mut x = a;
    while x > std::f64::consts::PI {
        x -= TAU;
    }
    while x < -std::f64::consts::PI {
        x += TAU;
    }
    x
}

impl HilbertTransform {
    pub fn new(window: usize, update_every: usize, fs: f64) -> Self {
        let w = next_pow2(window);
        Self {
            buf: RingBuffer::new(w, 0.0),
            plan: FftPlan::new(w),
            scratch: Vec::with_capacity(w),
            fs,
            update_every: update_every.max(1),
            since: 0,
            window: w,
            prev_phase: None,
        }
    }

    /// Returns `(amplitude, phase, instantaneous_frequency)` for the latest
    /// sample, or `None` while warming up / off-cadence.
    pub fn update(&mut self, x: f64) -> Option<(f64, f64, f64)> {
        self.buf.push(x);
        if !self.buf.is_full() {
            return None;
        }
        self.since += 1;
        if self.since < self.update_every {
            return None;
        }
        self.since = 0;
        let n = self.window;
        self.scratch.clear();
        for i in 0..n {
            self.scratch.push(Complex::new(*self.buf.get(i), 0.0));
        }
        self.plan.fft(&mut self.scratch);
        // Build the analytic signal: zero negative frequencies, double positive.
        let half = n / 2;
        for k in 1..half {
            self.scratch[k] = self.scratch[k].scale(2.0);
        }
        for k in (half + 1)..n {
            self.scratch[k] = Complex::new(0.0, 0.0);
        }
        self.plan.ifft(&mut self.scratch);
        // Evaluate the analytic signal at the window CENTER, where FFT-based
        // Hilbert edge artifacts are smallest (introduces a half-window delay).
        let mid = n / 2;
        let last = self.scratch[mid];
        let prev = self.scratch[mid - 1];
        let amplitude = last.abs();
        let phase = last.arg();
        let phase_prev = prev.arg();
        let dphi = wrap_pi(phase - phase_prev);
        let frequency = dphi / TAU * self.fs;
        self.prev_phase = Some(phase);
        Some((amplitude, phase, frequency))
    }

    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.scratch.clear();
        self.since = 0;
        self.prev_phase = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(n: usize, freq: f64) -> Vec<f64> {
        (0..n).map(|i| (TAU * freq * i as f64).sin()).collect()
    }

    #[test]
    fn stft_produces_frames_and_dominant() {
        let mut stft = ShortTimeFourierTransform::new(64, 32, 1.0, WindowKind::Hann, 8);
        let mut res = None;
        for x in sine(256, 0.1) {
            if let Some(r) = stft.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        assert!((r.dominant_frequency.unwrap() - 0.1).abs() < 0.04);
        let (frames, bins, vals) = stft.spectrogram();
        assert!(frames >= 1 && bins == 33 && vals.len() == frames * bins);
    }

    #[test]
    fn hilbert_envelope_of_sine_near_constant() {
        let mut h = HilbertTransform::new(128, 64, 1.0);
        let mut out = None;
        for x in sine(512, 0.05) {
            if let Some(r) = h.update(x) {
                out = Some(r);
            }
        }
        let (amp, _phase, freq) = out.unwrap();
        // Envelope of a unit sinusoid ~ 1; instantaneous freq ~ 0.05.
        assert!((amp - 1.0).abs() < 0.15, "amp {}", amp);
        assert!((freq - 0.05).abs() < 0.02, "freq {}", freq);
    }
}
