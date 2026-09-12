//! Wavelet transforms and wavelet-domain statistics: Haar discrete wavelet
//! transform (DWT), maximal-overlap DWT (MODWT), Morlet continuous wavelet
//! transform (CWT), wavelet variance/correlation/coherence, wavelet packets,
//! and multiresolution analysis. Block/window based (Tier B/C).

use crate::core::ring::RingBuffer;

/// Wavelet-domain result.
#[derive(Clone, Debug)]
pub struct WaveletResult {
    pub coefficients: Vec<f64>,
    pub scales: Option<Vec<f64>>,
}

fn haar_levels(window: usize) -> usize {
    let mut lv = 0;
    let mut n = window;
    while n > 1 {
        n >>= 1;
        lv += 1;
    }
    lv
}

/// Orthogonal Haar DWT. Returns coefficients ordered
/// `[approx_J, detail_J, ..., detail_1]` (length == input length).
fn haar_dwt(data: &[f64]) -> Vec<f64> {
    let n = data.len();
    if n < 2 {
        return data.to_vec();
    }
    let levels = haar_levels(n);
    let inv = 1.0 / 2.0f64.sqrt();
    let mut current = data.to_vec();
    let mut len = n;
    let mut details_rev: Vec<Vec<f64>> = Vec::new();
    for _ in 0..levels {
        if len < 2 {
            break;
        }
        let half = len / 2;
        let mut approx = vec![0.0; half];
        let mut detail = vec![0.0; half];
        for i in 0..half {
            approx[i] = (current[2 * i] + current[2 * i + 1]) * inv;
            detail[i] = (current[2 * i] - current[2 * i + 1]) * inv;
        }
        details_rev.push(detail);
        current = approx;
        len = half;
    }
    let mut coeffs = Vec::with_capacity(n);
    coeffs.extend_from_slice(&current);
    for d in details_rev.iter().rev() {
        coeffs.extend_from_slice(d);
    }
    coeffs
}

/// MODWT (a-trous, non-decimated) Haar detail coefficients per level.
/// Returns one vector per level (each of length `n`, circular).
fn modwt_levels(data: &[f64], levels: usize) -> Vec<Vec<f64>> {
    let n = data.len();
    let mut v = data.to_vec();
    let mut details = Vec::with_capacity(levels);
    for j in 0..levels {
        let step = 1usize << j;
        let mut w = vec![0.0; n];
        let mut vnew = vec![0.0; n];
        for t in 0..n {
            let a = v[t];
            let b = v[(t + n - step) % n];
            w[t] = 0.5 * (a - b);
            vnew[t] = 0.5 * (a + b);
        }
        details.push(w);
        v = vnew;
    }
    details
}

/// Haar DWT over a rolling power-of-two window (sliding, recomputed on
/// cadence). Output: full coefficient vector + dyadic scales.
#[derive(Clone, Debug)]
pub struct DiscreteWaveletTransform {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    update_every: usize,
    since: usize,
}

impl DiscreteWaveletTransform {
    pub fn new(window: usize, update_every: usize) -> Self {
        let w = window.next_power_of_two();
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            update_every: update_every.max(1),
            since: 0,
        }
    }
    pub fn update(&mut self, x: f64) -> Option<WaveletResult> {
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
        let coefficients = haar_dwt(&self.data);
        Some(WaveletResult { coefficients, scales: None })
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Multiresolution analysis: per-level MODWT detail coefficient at the latest
/// sample plus the coarsest approximation, with dyadic scales.
#[derive(Clone, Debug)]
pub struct MultiresolutionAnalysis {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    levels: usize,
    update_every: usize,
    since: usize,
}

impl MultiresolutionAnalysis {
    pub fn new(window: usize, levels: usize, update_every: usize) -> Self {
        let w = window.next_power_of_two();
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            levels,
            update_every: update_every.max(1),
            since: 0,
        }
    }
    pub fn update(&mut self, x: f64) -> Option<WaveletResult> {
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
        let details = modwt_levels(&self.data, self.levels);
        let n = self.data.len();
        let mut coefficients = Vec::with_capacity(self.levels);
        let mut scales = Vec::with_capacity(self.levels);
        for (j, w) in details.iter().enumerate() {
            coefficients.push(w[n - 1]);
            scales.push(2f64.powi((j + 1) as i32));
        }
        Some(WaveletResult { coefficients, scales: Some(scales) })
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// MODWT-based wavelet variance per scale (variance of detail coefficients).
#[derive(Clone, Debug)]
pub struct WaveletVariance {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    levels: usize,
    update_every: usize,
    since: usize,
}

impl WaveletVariance {
    pub fn new(window: usize, levels: usize, update_every: usize) -> Self {
        let w = window.next_power_of_two();
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            levels,
            update_every: update_every.max(1),
            since: 0,
        }
    }
    pub fn update(&mut self, x: f64) -> Option<WaveletResult> {
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
        let details = modwt_levels(&self.data, self.levels);
        let mut coefficients = Vec::with_capacity(self.levels);
        let mut scales = Vec::with_capacity(self.levels);
        for (j, w) in details.iter().enumerate() {
            let n = w.len() as f64;
            let mean = w.iter().sum::<f64>() / n;
            let var = w.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / n;
            coefficients.push(var);
            scales.push(2f64.powi((j + 1) as i32));
        }
        Some(WaveletResult { coefficients, scales: Some(scales) })
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Morlet continuous wavelet transform evaluated at the latest sample for a set
/// of scales. Output: coefficient magnitude per scale.
#[derive(Clone, Debug)]
pub struct CwtMorlet {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    scales: Vec<f64>,
    omega0: f64,
    update_every: usize,
    since: usize,
}

impl CwtMorlet {
    pub fn new(window: usize, scales: Vec<f64>, update_every: usize) -> Self {
        let w = window.next_power_of_two();
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            scales,
            omega0: 6.0,
            update_every: update_every.max(1),
            since: 0,
        }
    }
    pub fn update(&mut self, x: f64) -> Option<WaveletResult> {
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
        let n = self.data.len() as isize;
        let t = n - 1; // evaluate at the latest sample
        let norm = std::f64::consts::PI.powf(-0.25);
        let mut coefficients = Vec::with_capacity(self.scales.len());
        for &s in self.scales.iter() {
            let mut re = 0.0;
            let mut im = 0.0;
            for k in 0..n {
                let tau = (k - t) as f64 / s;
                let env = (-0.5 * tau * tau).exp() * norm / s.sqrt();
                let phase = self.omega0 * tau;
                // conj(psi) => exp(-i w0 tau)
                re += self.data[k as usize] * env * phase.cos();
                im -= self.data[k as usize] * env * phase.sin();
            }
            coefficients.push((re * re + im * im).sqrt());
        }
        Some(WaveletResult { coefficients, scales: Some(self.scales.clone()) })
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Haar wavelet packet decomposition (full binary tree) over a rolling window.
/// Returns the flattened final-level packet coefficients.
#[derive(Clone, Debug)]
pub struct WaveletPacket {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    levels: usize,
    update_every: usize,
    since: usize,
}

impl WaveletPacket {
    pub fn new(window: usize, levels: usize, update_every: usize) -> Self {
        let w = window.next_power_of_two();
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            levels,
            update_every: update_every.max(1),
            since: 0,
        }
    }
    pub fn update(&mut self, x: f64) -> Option<WaveletResult> {
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
        // Breadth-first packet decomposition.
        let mut nodes = vec![self.data.clone()];
        let inv = 1.0 / 2.0f64.sqrt();
        for _ in 0..self.levels {
            let mut next = Vec::with_capacity(nodes.len() * 2);
            for node in nodes.iter() {
                if node.len() < 2 {
                    next.push(node.clone());
                    next.push(node.clone());
                    continue;
                }
                let half = node.len() / 2;
                let mut a = vec![0.0; half];
                let mut d = vec![0.0; half];
                for i in 0..half {
                    a[i] = (node[2 * i] + node[2 * i + 1]) * inv;
                    d[i] = (node[2 * i] - node[2 * i + 1]) * inv;
                }
                next.push(a);
                next.push(d);
            }
            nodes = next;
        }
        let mut coefficients = Vec::new();
        for node in nodes.iter() {
            coefficients.extend_from_slice(node);
        }
        Some(WaveletResult { coefficients, scales: None })
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

/// Wavelet correlation / coherence between two streams across MODWT scales.
/// `mode = 0` -> correlation (Pearson of details), `mode = 1` -> coherence
/// (squared correlation).
#[derive(Clone, Debug)]
struct WaveletCrossCore {
    bufx: RingBuffer<f64>,
    bufy: RingBuffer<f64>,
    dx: Vec<f64>,
    dy: Vec<f64>,
    levels: usize,
    update_every: usize,
    since: usize,
}

impl WaveletCrossCore {
    fn new(window: usize, levels: usize, update_every: usize) -> Self {
        let w = window.next_power_of_two();
        Self {
            bufx: RingBuffer::new(w, 0.0),
            bufy: RingBuffer::new(w, 0.0),
            dx: Vec::with_capacity(w),
            dy: Vec::with_capacity(w),
            levels,
            update_every: update_every.max(1),
            since: 0,
        }
    }
    /// Returns per-level correlation, or None if not ready.
    fn correlations(&mut self) -> Option<Vec<f64>> {
        self.bufx.fill_vec(&mut self.dx);
        self.bufy.fill_vec(&mut self.dy);
        let wx = modwt_levels(&self.dx, self.levels);
        let wy = modwt_levels(&self.dy, self.levels);
        let mut out = Vec::with_capacity(self.levels);
        for j in 0..self.levels {
            let a = &wx[j];
            let b = &wy[j];
            let n = a.len() as f64;
            let ma = a.iter().sum::<f64>() / n;
            let mb = b.iter().sum::<f64>() / n;
            let mut sab = 0.0;
            let mut saa = 0.0;
            let mut sbb = 0.0;
            for i in 0..a.len() {
                let da = a[i] - ma;
                let db = b[i] - mb;
                sab += da * db;
                saa += da * da;
                sbb += db * db;
            }
            let denom = (saa * sbb).sqrt();
            out.push(if denom > 1e-18 { (sab / denom).clamp(-1.0, 1.0) } else { 0.0 });
        }
        Some(out)
    }
}

/// Wavelet correlation across scales (MODWT). Output: correlation per scale.
#[derive(Clone, Debug)]
pub struct WaveletCorrelation {
    core: WaveletCrossCore,
    ready: bool,
}

impl WaveletCorrelation {
    pub fn new(window: usize, levels: usize, update_every: usize) -> Self {
        Self { core: WaveletCrossCore::new(window, levels, update_every), ready: false }
    }
    pub fn update(&mut self, x: f64, y: f64) -> Option<crate::signal::spectral::SpectrumResult> {
        self.core.bufx.push(x);
        self.core.bufy.push(y);
        if !self.core.bufx.is_full() {
            return None;
        }
        self.core.since += 1;
        if self.core.since < self.core.update_every {
            return None;
        }
        self.core.since = 0;
        let corr = self.core.correlations()?;
        let scales: Vec<f64> = (0..corr.len()).map(|j| 2f64.powi((j + 1) as i32)).collect();
        Some(crate::signal::spectral::SpectrumResult {
            frequencies: scales,
            power: corr,
            dominant_frequency: None,
            peak_power: None,
        })
    }
    pub fn reset(&mut self) {
        self.core.bufx.clear(0.0);
        self.core.bufy.clear(0.0);
        self.core.since = 0;
        self.ready = false;
    }
}

/// Wavelet coherence across scales (squared MODWT correlation). Output in [0,1].
#[derive(Clone, Debug)]
pub struct WaveletCoherence {
    core: WaveletCrossCore,
}

impl WaveletCoherence {
    pub fn new(window: usize, levels: usize, update_every: usize) -> Self {
        Self { core: WaveletCrossCore::new(window, levels, update_every) }
    }
    pub fn update(&mut self, x: f64, y: f64) -> Option<crate::signal::spectral::SpectrumResult> {
        self.core.bufx.push(x);
        self.core.bufy.push(y);
        if !self.core.bufx.is_full() {
            return None;
        }
        self.core.since += 1;
        if self.core.since < self.core.update_every {
            return None;
        }
        self.core.since = 0;
        let corr = self.core.correlations()?;
        let coh: Vec<f64> = corr.iter().map(|c| c * c).collect();
        let scales: Vec<f64> = (0..coh.len()).map(|j| 2f64.powi((j + 1) as i32)).collect();
        Some(crate::signal::spectral::SpectrumResult {
            frequencies: scales,
            power: coh,
            dominant_frequency: None,
            peak_power: None,
        })
    }
    pub fn reset(&mut self) {
        self.core.bufx.clear(0.0);
        self.core.bufy.clear(0.0);
        self.core.since = 0;
    }
}

/// Maximal-overlap DWT detail coefficient at the latest sample per scale
/// (a compact streaming MODWT feature). Output: coefficients + scales.
#[derive(Clone, Debug)]
pub struct Modwt {
    buf: RingBuffer<f64>,
    data: Vec<f64>,
    levels: usize,
    update_every: usize,
    since: usize,
}

impl Modwt {
    pub fn new(window: usize, levels: usize, update_every: usize) -> Self {
        let w = window.next_power_of_two();
        Self {
            buf: RingBuffer::new(w, 0.0),
            data: Vec::with_capacity(w),
            levels,
            update_every: update_every.max(1),
            since: 0,
        }
    }
    pub fn update(&mut self, x: f64) -> Option<WaveletResult> {
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
        let details = modwt_levels(&self.data, self.levels);
        let n = self.data.len();
        let coefficients: Vec<f64> = details.iter().map(|w| w[n - 1]).collect();
        let scales: Vec<f64> = (0..self.levels).map(|j| 2f64.powi((j + 1) as i32)).collect();
        Some(WaveletResult { coefficients, scales: Some(scales) })
    }
    pub fn reset(&mut self) {
        self.buf.clear(0.0);
        self.data.clear();
        self.since = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haar_dwt_preserves_energy() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let c = haar_dwt(&data);
        let e_in: f64 = data.iter().map(|v| v * v).sum();
        let e_out: f64 = c.iter().map(|v| v * v).sum();
        assert!((e_in - e_out).abs() < 1e-9);
        assert_eq!(c.len(), data.len());
    }

    #[test]
    fn dwt_streaming_returns_coefficients() {
        let mut d = DiscreteWaveletTransform::new(8, 4);
        let mut got = None;
        for i in 0..16 {
            if let Some(r) = d.update(i as f64) {
                got = Some(r);
            }
        }
        assert_eq!(got.unwrap().coefficients.len(), 8);
    }

    #[test]
    fn modwt_levels_shape() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let d = modwt_levels(&data, 2);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].len(), 4);
    }

    #[test]
    fn wavelet_variance_positive() {
        let mut wv = WaveletVariance::new(16, 3, 8);
        let mut got = None;
        for i in 0..40 {
            if let Some(r) = wv.update((i as f64 * 0.3).sin()) {
                got = Some(r);
            }
        }
        let r = got.unwrap();
        assert_eq!(r.coefficients.len(), 3);
        assert!(r.coefficients.iter().all(|v| *v >= 0.0));
    }

    #[test]
    fn cwt_morlet_produces_coefficients() {
        let mut c = CwtMorlet::new(32, vec![2.0, 4.0, 8.0], 16);
        let mut got = None;
        for i in 0..64 {
            if let Some(r) = c.update((i as f64 * 0.2).sin()) {
                got = Some(r);
            }
        }
        let r = got.unwrap();
        assert_eq!(r.coefficients.len(), 3);
        assert!(r.coefficients.iter().all(|v| *v >= 0.0));
    }

    #[test]
    fn wavelet_coherence_identical_is_one() {
        let mut wc = WaveletCoherence::new(32, 3, 16);
        let mut got = None;
        for i in 0..80 {
            let v = (i as f64 * 0.2).sin();
            if let Some(r) = wc.update(v, v) {
                got = Some(r);
            }
        }
        let r = got.unwrap();
        assert!(r.power.iter().all(|c| (*c - 1.0).abs() < 1e-6));
    }

    #[test]
    fn wavelet_packet_expands() {
        let mut wp = WaveletPacket::new(8, 2, 4);
        let mut got = None;
        for i in 0..16 {
            if let Some(r) = wp.update(i as f64) {
                got = Some(r);
            }
        }
        // 2 levels of full packet decomposition of length-8 => 8 coefficients.
        assert_eq!(got.unwrap().coefficients.len(), 8);
    }
}
