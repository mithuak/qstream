//! Window functions for spectral estimation, including DPSS (Slepian) tapers
//! computed from the prolate spheroidal matrix via the Jacobi eigensolver.

use super::matrix::{jacobi_eigen, DMat};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowKind {
    Rectangular,
    Hann,
    Hamming,
    Blackman,
    BlackmanHarris,
}

impl WindowKind {
    pub fn parse(s: &str) -> Option<WindowKind> {
        match s.to_ascii_lowercase().as_str() {
            "rect" | "rectangular" | "boxcar" | "none" => Some(WindowKind::Rectangular),
            "hann" | "hanning" => Some(WindowKind::Hann),
            "hamming" => Some(WindowKind::Hamming),
            "blackman" => Some(WindowKind::Blackman),
            "blackmanharris" | "blackman-harris" => Some(WindowKind::BlackmanHarris),
            _ => None,
        }
    }
}

/// Fill `out` (length `n`) with the given window.
pub fn fill_window(kind: WindowKind, n: usize, out: &mut Vec<f64>) {
    out.clear();
    out.reserve(n);
    if n == 0 {
        return;
    }
    let denom = (n - 1).max(1) as f64;
    for i in 0..n {
        let x = std::f64::consts::PI * 2.0 * i as f64 / denom;
        let w = match kind {
            WindowKind::Rectangular => 1.0,
            WindowKind::Hann => 0.5 * (1.0 - x.cos()),
            WindowKind::Hamming => 0.54 - 0.46 * x.cos(),
            WindowKind::Blackman => 0.42 - 0.5 * x.cos() + 0.08 * (2.0 * x).cos(),
            WindowKind::BlackmanHarris => {
                0.35875 - 0.48829 * x.cos() + 0.14128 * (2.0 * x).cos()
                    - 0.01168 * (3.0 * x).cos()
            }
        };
        out.push(w);
    }
}

/// Coherent gain (sum of the window), used for amplitude normalization.
pub fn coherent_gain(win: &[f64]) -> f64 {
    win.iter().sum()
}

/// Noise power bandwidth normalization factor: sum(w^2) / (sum(w))^2 * n.
/// Returns the ENBW scaling used to normalize a periodogram.
pub fn power_sum(win: &[f64]) -> f64 {
    win.iter().map(|w| w * w).sum()
}

/// Compute `k` DPSS (Slepian) tapers of length `n` for half-bandwidth `nw`
/// (time-bandwidth product). Tapers are returned as the leading eigenvectors
/// of the prolate matrix, ordered by decreasing eigenvalue (concentration).
///
/// This is O(n^2) memory and O(n^3) time; it is intended for Tier C
/// multitaper methods and should be computed once and cached by the caller.
pub fn dpss(n: usize, nw: f64, k: usize) -> Vec<Vec<f64>> {
    assert!(n > 0 && k > 0 && k <= n);
    let w = nw / n as f64; // half-bandwidth in cycles/sample
    // Prolate matrix D[p][q] = sin(2*pi*w*(p-q)) / (pi*(p-q)), D[p][p] = 2w.
    let mut a = DMat::zeros(n, n);
    for p in 0..n {
        for q in 0..n {
            let d = (p as i64 - q as i64) as f64;
            let val = if p == q {
                2.0 * w
            } else {
                (2.0 * std::f64::consts::PI * w * d).sin() / (std::f64::consts::PI * d)
            };
            a.set(p, q, val);
        }
    }
    let (vals, vecs) = jacobi_eigen(&a, 100);
    let mut tapers = Vec::with_capacity(k);
    for j in 0..k.min(n) {
        let mut t: Vec<f64> = (0..n).map(|r| vecs.get(r, j)).collect();
        // Normalize to unit energy.
        let norm: f64 = t.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for x in t.iter_mut() {
                *x /= norm;
            }
        }
        // Deterministic sign convention: make the largest-magnitude entry positive.
        if let Some(idx) = t.iter().enumerate().max_by(|a, b| {
            a.1.abs().partial_cmp(&b.1.abs()).unwrap()
        }).map(|(i, _)| i) {
            if t[idx] < 0.0 {
                for x in t.iter_mut() {
                    *x = -*x;
                }
            }
        }
        let _ = vals[j];
        tapers.push(t);
    }
    tapers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hann_symmetric() {
        let mut w = Vec::new();
        fill_window(WindowKind::Hann, 5, &mut w);
        assert!(w[0].abs() < 1e-12);
        assert!((w[2] - 1.0).abs() < 1e-12);
        assert!((w[1] - w[3]).abs() < 1e-12);
    }

    #[test]
    fn dpss_first_taper_positive_energy() {
        let t = dpss(16, 2.0, 2);
        assert_eq!(t.len(), 2);
        let e: f64 = t[0].iter().map(|x| x * x).sum();
        assert!((e - 1.0).abs() < 1e-9);
    }
}
