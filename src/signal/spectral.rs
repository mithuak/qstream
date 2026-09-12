//! Spectral estimators: Goertzel, periodogram, Welch PSD, FFT spectral density,
//! Bartlett, AR spectrum (Burg/Yule-Walker), Blackman-Tukey, Thomson multitaper,
//! cross-spectrum, coherence, and spectral-shape features.
//!
//! Heavy estimators (Tier C) push every observation into a ring buffer but only
//! recompute the spectrum every `update_every` samples, returning `None` on
//! ticks where no computation ran.

use crate::core::fft::{next_pow2, FftPlan};
use crate::core::num::Complex;
use crate::core::ring::RingBuffer;
use crate::core::window::{dpss, fill_window, WindowKind};
use crate::signal::prediction::{autocorrelation, burg_ar};

/// One-sided spectral estimate.
#[derive(Clone, Debug)]
pub struct SpectrumResult {
    pub frequencies: Vec<f64>,
    pub power: Vec<f64>,
    pub dominant_frequency: Option<f64>,
    pub peak_power: Option<f64>,
}

impl SpectrumResult {
    pub(crate) fn from_psd(frequencies: Vec<f64>, power: Vec<f64>) -> Self {
        let (dominant_frequency, peak_power) = dominant(&frequencies, &power);
        Self { frequencies, power, dominant_frequency, peak_power }
    }
}

/// Find the dominant (peak) frequency ignoring the DC bin.
pub fn dominant(freqs: &[f64], power: &[f64]) -> (Option<f64>, Option<f64>) {
    if power.is_empty() {
        return (None, None);
    }
    let mut best = 0usize;
    let mut bestv = f64::NEG_INFINITY;
    for k in 1..power.len() {
        if power[k] > bestv {
            bestv = power[k];
            best = k;
        }
    }
    if best == 0 && power.len() == 1 {
        (Some(freqs[0]), Some(power[0]))
    } else {
        (Some(freqs[best]), Some(bestv))
    }
}

/// Compute the one-sided power spectral density of `data` (length N, power of
/// two) using a precomputed window and FFT plan. Results are written into
/// `freqs` and `psd` (resized). Density scaling: `2*|X_k|^2 / (fs * sum(w^2))`.
fn one_sided_psd(
    plan: &FftPlan,
    win: &[f64],
    data: &[f64],
    fs: f64,
    scratch: &mut Vec<Complex>,
    bins: &mut Vec<Complex>,
    freqs: &mut Vec<f64>,
    psd: &mut Vec<f64>,
) {
    let n = data.len();
    let win_power: f64 = win.iter().map(|w| w * w).sum();
    let denom = fs * win_power.max(1e-30);
    // Apply window into scratch (reuse as f64 via bins later); do it in place.
    scratch.clear();
    scratch.reserve(n);
    for i in 0..n {
        scratch.push(Complex::new(data[i] * win[i], 0.0));
    }
    // FFT in place.
    plan.fft(scratch);
    let bins_n = n / 2 + 1;
    freqs.clear();
    psd.clear();
    freqs.reserve(bins_n);
    psd.reserve(bins_n);
    for k in 0..bins_n {
        let c = scratch[k];
        let p = c.norm_sqr();
        let scale = if k == 0 || k == n / 2 { 1.0 } else { 2.0 };
        psd.push(scale * p / denom);
        freqs.push(k as f64 * fs / n as f64);
    }
    let _ = bins;
}

/// Shared streaming Welch/periodogram core.
#[derive(Clone, Debug)]
struct SpectralCore {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    plan: FftPlan,
    win: Vec<f64>,
    scratch: Vec<Complex>,
    bins: Vec<Complex>,
    fs: f64,
    update_every: usize,
    since: usize,
    psd_sum: Vec<f64>,
    psd_seg: Vec<f64>,
    freqs: Vec<f64>,
    seg_count: usize,
    average: bool,
}

impl SpectralCore {
    fn new(window: usize, fs: f64, update_every: usize, window_kind: WindowKind, average: bool) -> Self {
        let w = next_pow2(window);
        let mut win = Vec::new();
        fill_window(window_kind, w, &mut win);
        let plan = FftPlan::new(w);
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            plan,
            win,
            scratch: Vec::with_capacity(w),
            bins: Vec::new(),
            fs,
            update_every: update_every.max(1),
            since: 0,
            psd_sum: Vec::new(),
            psd_seg: Vec::new(),
            freqs: Vec::new(),
            seg_count: 0,
            average,
        }
    }

    fn push(&mut self, x: f64) -> Option<SpectrumResult> {
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
        one_sided_psd(
            &self.plan,
            &self.win,
            &self.data,
            self.fs,
            &mut self.scratch,
            &mut self.bins,
            &mut self.freqs,
            &mut self.psd_seg,
        );
        if self.average {
            if self.psd_sum.len() != self.psd_seg.len() {
                self.psd_sum = vec![0.0; self.psd_seg.len()];
            }
            for (a, b) in self.psd_sum.iter_mut().zip(self.psd_seg.iter()) {
                *a += *b;
            }
            self.seg_count += 1;
            let inv = 1.0 / self.seg_count as f64;
            let power: Vec<f64> = self.psd_sum.iter().map(|v| v * inv).collect();
            Some(SpectrumResult::from_psd(self.freqs.clone(), power))
        } else {
            Some(SpectrumResult::from_psd(self.freqs.clone(), self.psd_seg.clone()))
        }
    }

    fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.psd_sum.clear();
        self.psd_seg.clear();
        self.freqs.clear();
        self.seg_count = 0;
        self.since = 0;
    }
}

/// Welch overlapped-periodogram PSD (averaged segments). Tier C.
#[derive(Clone, Debug)]
pub struct WelchPsd {
    core: SpectralCore,
}

impl WelchPsd {
    pub fn new(window: usize, update_every: usize) -> Self {
        Self::with_options(window, update_every, 1.0, WindowKind::Hann)
    }
    pub fn with_options(window: usize, update_every: usize, fs: f64, kind: WindowKind) -> Self {
        Self { core: SpectralCore::new(window, fs, update_every, kind, true) }
    }
    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
        self.core.push(x)
    }
    pub fn reset(&mut self) {
        self.core.reset();
    }
}

/// Single-window periodogram (`|FFT|^2` density), recomputed on cadence.
#[derive(Clone, Debug)]
pub struct Periodogram {
    core: SpectralCore,
}

impl Periodogram {
    pub fn new(window: usize, update_every: usize) -> Self {
        Self::with_options(window, update_every, 1.0, WindowKind::Rectangular)
    }
    pub fn with_options(window: usize, update_every: usize, fs: f64, kind: WindowKind) -> Self {
        Self { core: SpectralCore::new(window, fs, update_every, kind, false) }
    }
    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
        self.core.push(x)
    }
    pub fn reset(&mut self) {
        self.core.reset();
    }
}

/// FFT spectral density estimation (windowed periodogram with Hann taper).
#[derive(Clone, Debug)]
pub struct FftSpectralDensity {
    core: SpectralCore,
}

impl FftSpectralDensity {
    pub fn new(window: usize, update_every: usize) -> Self {
        Self::with_options(window, update_every, 1.0, WindowKind::Hann)
    }
    pub fn with_options(window: usize, update_every: usize, fs: f64, kind: WindowKind) -> Self {
        Self { core: SpectralCore::new(window, fs, update_every, kind, false) }
    }
    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
        self.core.push(x)
    }
    pub fn reset(&mut self) {
        self.core.reset();
    }
}

/// Bartlett's method: average periodograms of successive (non-overlapping)
/// segments. The streaming core averages every recomputed segment.
#[derive(Clone, Debug)]
pub struct BartlettMethod {
    core: SpectralCore,
}

impl BartlettMethod {
    pub fn new(segment: usize) -> Self {
        Self::with_options(segment, segment, 1.0, WindowKind::Rectangular)
    }
    pub fn with_options(segment: usize, update_every: usize, fs: f64, kind: WindowKind) -> Self {
        Self { core: SpectralCore::new(segment, fs, update_every, kind, true) }
    }
    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
        self.core.push(x)
    }
    pub fn reset(&mut self) {
        self.core.reset();
    }
}

/// Goertzel algorithm: recursive DFT bins at a set of target normalized
/// frequencies, evaluated every `block` samples. Tier A per-frequency work.
#[derive(Clone, Debug)]
pub struct Goertzel {
    freqs: Vec<f64>,
    coeff: Vec<f64>,
    s1: Vec<f64>,
    s2: Vec<f64>,
    block: usize,
    count: usize,
}

impl Goertzel {
    /// `freqs` are normalized frequencies in cycles/sample (0..0.5).
    pub fn new(freqs: Vec<f64>, block: usize) -> Self {
        assert!(block > 1, "block must be > 1");
        let coeff: Vec<f64> = freqs
            .iter()
            .map(|f| 2.0 * (2.0 * std::f64::consts::PI * f).cos())
            .collect();
        let n = freqs.len();
        Self { freqs, coeff, s1: vec![0.0; n], s2: vec![0.0; n], block, count: 0 }
    }
    pub fn update(&mut self, x: f64) -> Option<SpectrumResult> {
        for i in 0..self.freqs.len() {
            let s0 = x + self.coeff[i] * self.s1[i] - self.s2[i];
            self.s2[i] = self.s1[i];
            self.s1[i] = s0;
        }
        self.count += 1;
        if self.count < self.block {
            return None;
        }
        self.count = 0;
        let mut power = Vec::with_capacity(self.freqs.len());
        for i in 0..self.freqs.len() {
            let p = self.s1[i] * self.s1[i] + self.s2[i] * self.s2[i]
                - self.coeff[i] * self.s1[i] * self.s2[i];
            power.push((p / (self.block as f64 * self.block as f64)).max(0.0));
            self.s1[i] = 0.0;
            self.s2[i] = 0.0;
        }
        Some(SpectrumResult::from_psd(self.freqs.clone(), power))
    }
    pub fn reset(&mut self) {
        for i in 0..self.freqs.len() {
            self.s1[i] = 0.0;
            self.s2[i] = 0.0;
        }
        self.count = 0;
    }
}

/// Autoregressive spectral density via Burg AR coefficients, evaluated on a
/// frequency grid. Recomputed on cadence over a rolling window. Tier C.
#[derive(Clone, Debug)]
pub struct ArSpectrum {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    order: usize,
    nfft: usize,
    fs: f64,
    update_every: usize,
    since: usize,
}

impl ArSpectrum {
    pub fn new(window: usize, order: usize, nfft: usize, update_every: usize) -> Self {
        let nfft = next_pow2(nfft);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            order,
            nfft,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
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
        let (a, _) = burg_ar(&self.data, self.order);
        // Residual variance via autocorrelation and AR coefficients.
        let acf = autocorrelation(&self.data, self.order);
        let mut var = acf[0];
        for i in 0..self.order {
            var -= a[i] * acf[i + 1];
        }
        let var = var.max(1e-18);
        let bins = self.nfft / 2 + 1;
        let mut freqs = Vec::with_capacity(bins);
        let mut power = Vec::with_capacity(bins);
        for k in 0..bins {
            let f = k as f64 * self.fs / self.nfft as f64;
            let w = 2.0 * std::f64::consts::PI * f;
            // |1 - sum a_i e^{-j w i}|^2
            let mut re = 1.0;
            let mut im = 0.0;
            for i in 0..self.order {
                let ang = w * (i + 1) as f64;
                re -= a[i] * ang.cos();
                im += a[i] * ang.sin();
            }
            let denom = (re * re + im * im).max(1e-18);
            freqs.push(f);
            power.push(var / (denom * self.fs));
        }
        Some(SpectrumResult::from_psd(freqs, power))
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Blackman-Tukey spectral estimation: FFT of a lag-windowed autocorrelation.
#[derive(Clone, Debug)]
pub struct BlackmanTukey {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    max_lag: usize,
    nfft: usize,
    fs: f64,
    update_every: usize,
    since: usize,
}

impl BlackmanTukey {
    pub fn new(window: usize, max_lag: usize, update_every: usize) -> Self {
        let nfft = next_pow2(2 * max_lag);
        Self {
            buf: RingBuffer::new(window, 0.0),
            data: Vec::with_capacity(window),
            max_lag,
            nfft,
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
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
        let acf = autocorrelation(&self.data, self.max_lag);
        // Build symmetric lag-windowed sequence (Bartlett lag window), length nfft.
        let mut seq = vec![0.0f64; self.nfft];
        for lag in 0..=self.max_lag {
            let w = 1.0 - lag as f64 / (self.max_lag as f64 + 1.0); // Bartlett lag window
            let val = acf[lag] * w;
            seq[lag] = val;
            if lag > 0 {
                seq[self.nfft - lag] = val;
            }
        }
        let plan = FftPlan::new(self.nfft);
        let mut scratch: Vec<Complex> = seq.iter().map(|v| Complex::new(*v, 0.0)).collect();
        plan.fft(&mut scratch);
        let bins = self.nfft / 2 + 1;
        let mut freqs = Vec::with_capacity(bins);
        let mut power = Vec::with_capacity(bins);
        for k in 0..bins {
            // Real spectrum: take real part (imaginary ~ 0 for symmetric input).
            let p = scratch[k].re / self.fs;
            freqs.push(k as f64 * self.fs / self.nfft as f64);
            power.push(p.max(0.0));
        }
        Some(SpectrumResult::from_psd(freqs, power))
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Thomson multitaper PSD using DPSS (Slepian) tapers. Tapers are computed once
/// (cached) and the eigenspectra averaged. Tier C.
#[derive(Clone, Debug)]
pub struct MultitaperPsd {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    plan: FftPlan,
    tapers: Vec<Vec<f64>>,
    scratch: Vec<Complex>,
    windowed: Vec<f64>,
    fs: f64,
    update_every: usize,
    since: usize,
}

impl MultitaperPsd {
    /// `nw` time-bandwidth product (e.g. 3.0), `tapers` number of DPSS (e.g. 5).
    pub fn new(window: usize, nw: f64, tapers: usize, update_every: usize) -> Self {
        let w = next_pow2(window);
        let plan = FftPlan::new(w);
        let tapers = dpss(w, nw, tapers);
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            plan,
            tapers,
            scratch: Vec::with_capacity(w),
            windowed: vec![0.0; w],
            fs: 1.0,
            update_every: update_every.max(1),
            since: 0,
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
        let bins = n / 2 + 1;
        let mut acc = vec![0.0f64; bins];
        for taper in self.tapers.iter() {
            for i in 0..n {
                self.windowed[i] = self.data[i] * taper[i];
            }
            self.scratch.clear();
            for i in 0..n {
                self.scratch.push(Complex::new(self.windowed[i], 0.0));
            }
            self.plan.fft(&mut self.scratch);
            for k in 0..bins {
                let scale = if k == 0 || k == n / 2 { 1.0 } else { 2.0 };
                acc[k] += scale * self.scratch[k].norm_sqr();
            }
        }
        let k = self.tapers.len().max(1) as f64;
        let mut freqs = Vec::with_capacity(bins);
        let mut power = Vec::with_capacity(bins);
        for i in 0..bins {
            freqs.push(i as f64 * self.fs / n as f64);
            power.push(acc[i] / (k * self.fs));
        }
        Some(SpectrumResult::from_psd(freqs, power))
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Two-signal cross-spectral core (Welch-averaged), used by CrossSpectrum and
/// Coherence.
#[derive(Clone, Debug)]
struct CrossCore {
    bufx: RingBuffer<f64>,
    bufy: RingBuffer<f64>,
    dx: Vec<f64>,
    dy: Vec<f64>,
    plan: FftPlan,
    win: Vec<f64>,
    sx: Vec<Complex>,
    sy: Vec<Complex>,
    fs: f64,
    update_every: usize,
    since: usize,
    // accumulated spectra
    pxx: Vec<f64>,
    pyy: Vec<f64>,
    pxy_re: Vec<f64>,
    pxy_im: Vec<f64>,
    freqs: Vec<f64>,
    seg_count: usize,
}

impl CrossCore {
    fn new(window: usize, fs: f64, update_every: usize, kind: WindowKind) -> Self {
        let w = next_pow2(window);
        let mut win = Vec::new();
        fill_window(kind, w, &mut win);
        Self {
            bufx: RingBuffer::new(w, 0.0),
            bufy: RingBuffer::new(w, 0.0),
            dx: Vec::with_capacity(w),
            dy: Vec::with_capacity(w),
            plan: FftPlan::new(w),
            win,
            sx: Vec::with_capacity(w),
            sy: Vec::with_capacity(w),
            fs,
            update_every: update_every.max(1),
            since: 0,
            pxx: Vec::new(),
            pyy: Vec::new(),
            pxy_re: Vec::new(),
            pxy_im: Vec::new(),
            freqs: Vec::new(),
            seg_count: 0,
        }
    }

    fn push(&mut self, x: f64, y: f64) -> bool {
        self.bufx.push(x);
        self.bufy.push(y);
        if !self.bufx.is_full() {
            return false;
        }
        self.since += 1;
        if self.since < self.update_every {
            return false;
        }
        self.since = 0;
        self.compute();
        true
    }

    fn compute(&mut self) {
        self.bufx.fill_vec(&mut self.dx);
        self.bufy.fill_vec(&mut self.dy);
        let n = self.dx.len();
        let win_power: f64 = self.win.iter().map(|w| w * w).sum();
        let denom = self.fs * win_power.max(1e-30);
        self.sx.clear();
        self.sy.clear();
        for i in 0..n {
            self.sx.push(Complex::new(self.dx[i] * self.win[i], 0.0));
            self.sy.push(Complex::new(self.dy[i] * self.win[i], 0.0));
        }
        self.plan.fft(&mut self.sx);
        self.plan.fft(&mut self.sy);
        let bins = n / 2 + 1;
        if self.pxx.len() != bins {
            self.pxx = vec![0.0; bins];
            self.pyy = vec![0.0; bins];
            self.pxy_re = vec![0.0; bins];
            self.pxy_im = vec![0.0; bins];
            self.freqs = (0..bins).map(|k| k as f64 * self.fs / n as f64).collect();
        }
        for k in 0..bins {
            let scale = if k == 0 || k == n / 2 { 1.0 } else { 2.0 };
            self.pxx[k] += scale * self.sx[k].norm_sqr() / denom;
            self.pyy[k] += scale * self.sy[k].norm_sqr() / denom;
            let cross = self.sx[k] * self.sy[k].conj();
            self.pxy_re[k] += scale * cross.re / denom;
            self.pxy_im[k] += scale * cross.im / denom;
        }
        self.seg_count += 1;
    }

    fn reset(&mut self) {
        self.bufx.clear(0.0);
        self.bufy.clear(0.0);
        self.dx.clear();
        self.dy.clear();
        self.pxx.clear();
        self.pyy.clear();
        self.pxy_re.clear();
        self.pxy_im.clear();
        self.freqs.clear();
        self.seg_count = 0;
        self.since = 0;
    }
}

/// Cross-spectral density magnitude between two streams (Welch-averaged).
#[derive(Clone, Debug)]
pub struct CrossSpectrum {
    core: CrossCore,
}

impl CrossSpectrum {
    pub fn new(window: usize, update_every: usize) -> Self {
        Self::with_options(window, update_every, 1.0, WindowKind::Hann)
    }
    pub fn with_options(window: usize, update_every: usize, fs: f64, kind: WindowKind) -> Self {
        Self { core: CrossCore::new(window, fs, update_every, kind) }
    }
    pub fn update(&mut self, x: f64, y: f64) -> Option<SpectrumResult> {
        if !self.core.push(x, y) {
            return None;
        }
        let c = self.core.seg_count as f64;
        let power: Vec<f64> = (0..self.core.pxy_re.len())
            .map(|k| {
                let re = self.core.pxy_re[k] / c;
                let im = self.core.pxy_im[k] / c;
                (re * re + im * im).sqrt()
            })
            .collect();
        Some(SpectrumResult::from_psd(self.core.freqs.clone(), power))
    }
    pub fn reset(&mut self) {
        self.core.reset();
    }
}

/// Magnitude-squared coherence between two streams (Welch-averaged):
/// `|Pxy|^2 / (Pxx * Pyy)`, values in [0,1].
#[derive(Clone, Debug)]
pub struct Coherence {
    core: CrossCore,
}

impl Coherence {
    pub fn new(window: usize, update_every: usize) -> Self {
        Self::with_options(window, update_every, 1.0, WindowKind::Hann)
    }
    pub fn with_options(window: usize, update_every: usize, fs: f64, kind: WindowKind) -> Self {
        Self { core: CrossCore::new(window, fs, update_every, kind) }
    }
    pub fn update(&mut self, x: f64, y: f64) -> Option<SpectrumResult> {
        if !self.core.push(x, y) {
            return None;
        }
        let c = self.core.seg_count as f64;
        let power: Vec<f64> = (0..self.core.pxy_re.len())
            .map(|k| {
                let re = self.core.pxy_re[k] / c;
                let im = self.core.pxy_im[k] / c;
                let pxx = self.core.pxx[k] / c;
                let pyy = self.core.pyy[k] / c;
                let denom = pxx * pyy;
                if denom <= 1e-30 {
                    0.0
                } else {
                    ((re * re + im * im) / denom).clamp(0.0, 1.0)
                }
            })
            .collect();
        let (dominant_frequency, peak_power) = dominant(&self.core.freqs, &power);
        Some(SpectrumResult {
            frequencies: self.core.freqs.clone(),
            power,
            dominant_frequency,
            peak_power,
        })
    }
    pub fn reset(&mut self) {
        self.core.reset();
    }
}

/// Spectral-shape features computed from a one-sided PSD.
#[derive(Clone, Debug)]
pub struct SpectralShape {
    pub centroid: f64,
    pub bandwidth: f64,
    pub entropy: f64,
    pub rolloff: f64,
    pub flatness: f64,
    pub dominant_frequency: f64,
}

/// Compute spectral-shape features from a [`SpectrumResult`].
pub fn spectral_shape(spectrum: &SpectrumResult) -> SpectralShape {
    let f = &spectrum.frequencies;
    let p = &spectrum.power;
    let n = p.len();
    if n == 0 {
        return SpectralShape {
            centroid: 0.0,
            bandwidth: 0.0,
            entropy: 0.0,
            rolloff: 0.0,
            flatness: 0.0,
            dominant_frequency: 0.0,
        };
    }
    let total: f64 = p.iter().sum();
    let mut centroid = 0.0;
    if total > 0.0 {
        for k in 0..n {
            centroid += f[k] * p[k];
        }
        centroid /= total;
    }
    let mut bandwidth = 0.0;
    if total > 0.0 {
        for k in 0..n {
            let d = f[k] - centroid;
            bandwidth += d * d * p[k];
        }
        bandwidth = (bandwidth / total).sqrt();
    }
    // Spectral entropy (normalized).
    let mut entropy = 0.0;
    if total > 0.0 {
        for k in 0..n {
            let pk = p[k] / total;
            if pk > 0.0 {
                entropy -= pk * pk.ln();
            }
        }
        let norm = (n as f64).ln();
        if norm > 0.0 {
            entropy /= norm;
        }
    }
    // Rolloff frequency (85% energy).
    let mut rolloff = 0.0;
    if total > 0.0 {
        let mut acc = 0.0;
        for k in 0..n {
            acc += p[k];
            if acc >= 0.85 * total {
                rolloff = f[k];
                break;
            }
        }
    }
    // Spectral flatness: geometric mean / arithmetic mean.
    let mut logsum = 0.0;
    let mut pos = 0usize;
    for k in 0..n {
        if p[k] > 0.0 {
            logsum += p[k].ln();
            pos += 1;
        }
    }
    let flatness = if pos == n && pos > 0 && total > 0.0 {
        let geo = (logsum / pos as f64).exp();
        let ari = total / n as f64;
        geo / ari
    } else {
        0.0
    };
    let dominant_frequency = spectrum.dominant_frequency.unwrap_or(0.0);
    SpectralShape { centroid, bandwidth, entropy, rolloff, flatness, dominant_frequency }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(n: usize, freq: f64) -> Vec<f64> {
        (0..n).map(|i| (2.0 * std::f64::consts::PI * freq * i as f64).sin()).collect()
    }

    #[test]
    fn welch_finds_tone() {
        let mut w = WelchPsd::new(64, 16);
        let sig = sine(256, 0.1); // bin ~ 0.1*64 = 6.4
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = w.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        let df = r.dominant_frequency.unwrap();
        assert!((df - 0.1).abs() < 0.03, "dominant {} vs 0.1", df);
    }

    #[test]
    fn periodogram_returns_spectrum() {
        let mut p = Periodogram::new(32, 8);
        let sig = sine(128, 0.2);
        let mut got = None;
        for &x in sig.iter() {
            if let Some(r) = p.update(x) {
                got = Some(r);
            }
        }
        let r = got.unwrap();
        assert_eq!(r.frequencies.len(), r.power.len());
        assert!((r.dominant_frequency.unwrap() - 0.2).abs() < 0.06);
    }

    #[test]
    fn goertzel_detects_target() {
        let mut g = Goertzel::new(vec![0.1, 0.3], 64);
        let sig = sine(128, 0.1);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = g.update(x) {
                res = Some(r);
            }
        }
        let r = res.unwrap();
        // Power at 0.1 should dominate 0.3.
        assert!(r.power[0] > r.power[1]);
    }

    #[test]
    fn ar_spectrum_positive() {
        let mut ar = ArSpectrum::new(128, 8, 64, 32);
        let sig = sine(256, 0.15);
        let mut got = None;
        for &x in sig.iter() {
            if let Some(r) = ar.update(x) {
                got = Some(r);
            }
        }
        let r = got.unwrap();
        assert!(r.power.iter().all(|p| *p >= 0.0));
        assert!((r.dominant_frequency.unwrap() - 0.15).abs() < 0.05);
    }

    #[test]
    fn multitaper_runs() {
        let mut mt = MultitaperPsd::new(64, 3.0, 4, 32);
        let sig = sine(128, 0.1);
        let mut got = None;
        for &x in sig.iter() {
            if let Some(r) = mt.update(x) {
                got = Some(r);
            }
        }
        assert!(got.is_some());
        let r = got.unwrap();
        assert!(r.power.iter().all(|p| *p >= 0.0));
    }

    #[test]
    fn coherence_identical_signals_is_one() {
        let mut coh = Coherence::new(64, 32);
        let sig = sine(256, 0.1);
        let mut got = None;
        for i in 0..sig.len() {
            if let Some(r) = coh.update(sig[i], sig[i]) {
                got = Some(r);
            }
        }
        let r = got.unwrap();
        // Coherence of a signal with itself ~ 1 at all bins.
        let mean: f64 = r.power.iter().sum::<f64>() / r.power.len() as f64;
        assert!(mean > 0.9, "mean coherence {}", mean);
    }

    #[test]
    fn spectral_shape_features_valid() {
        let mut w = WelchPsd::new(64, 32);
        let sig = sine(256, 0.1);
        let mut res = None;
        for &x in sig.iter() {
            if let Some(r) = w.update(x) {
                res = Some(r);
            }
        }
        let shape = spectral_shape(&res.unwrap());
        assert!(shape.entropy >= 0.0 && shape.entropy <= 1.0000001);
        assert!(shape.flatness >= 0.0 && shape.flatness <= 1.0000001);
        assert!(shape.centroid > 0.0);
    }

    #[test]
    fn blackman_tukey_runs() {
        let mut bt = BlackmanTukey::new(128, 16, 32);
        let sig = sine(256, 0.1);
        let mut got = None;
        for &x in sig.iter() {
            if let Some(r) = bt.update(x) {
                got = Some(r);
            }
        }
        assert!(got.is_some());
    }
}
