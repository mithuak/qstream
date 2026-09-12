//! Pure-Rust latency/throughput benchmark (design section 13).
//!
//! Run with: `cargo run --release --example bench`
//!
//! Measures per-update cost without the Python boundary, reporting mean ns per
//! update, throughput, and p50/p95/p99 batch latency.

use std::time::Instant;

use _qstream::core::traits::ScalarIndicator;
use _qstream::finance::volatility::{EwmaVolatility, Garch11, RollingVolatility};
use _qstream::signal::kalman::{ConstantVelocityKalman, LmsFilter};
use _qstream::signal::regime::{PageHinkley, TeagerKaiser};
use _qstream::signal::spectral::WelchPsd;

const N: usize = 1_000_000;
const BATCH: usize = 1_000;

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx]
}

/// Benchmark a scalar update closure. Returns (mean_ns, p50, p95, p99) per update.
fn bench_scalar<F: FnMut(f64)>(label: &str, mut f: F) {
    let data: Vec<f64> = (0..N).map(|i| ((i % 97) as f64 - 48.0) * 0.001).collect();
    // Warm up.
    for &x in data.iter().take(1000) {
        f(x);
    }
    let mut batch_times: Vec<f64> = Vec::with_capacity(N / BATCH);
    let start = Instant::now();
    let mut chunks = 0usize;
    for chunk in data.chunks(BATCH) {
        let t0 = Instant::now();
        for &x in chunk {
            f(x);
        }
        batch_times.push(t0.elapsed().as_nanos() as f64 / chunk.len() as f64);
        chunks += 1;
    }
    let total = start.elapsed().as_nanos() as f64;
    let mean_ns = total / N as f64;
    batch_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = percentile(&batch_times, 50.0);
    let p95 = percentile(&batch_times, 95.0);
    let p99 = percentile(&batch_times, 99.0);
    let throughput = 1e9 / mean_ns;
    println!(
        "{label:<24} mean={mean_ns:8.2} ns  p50={p50:8.2}  p95={p95:8.2}  p99={p99:8.2}  thr={throughput:10.0}/s  (chunks={chunks})"
    );
}

fn main() {
    println!("qstream pure-Rust update benchmark (N={N}, batch={BATCH})");
    println!("---------------------------------------------------------------");

    bench_scalar("EwmaVolatility", |x| {
        let mut ind = EwmaVolatility::risk_metrics();
        let _ = std::hint::black_box(ScalarIndicator::update(&mut ind, x));
    });

    // Stateful indicators need to persist across updates; use a closure that
    // captures the indicator by mutable reference.
    let mut g = Garch11::default_params();
    bench_scalar("Garch11", |x| {
        let _ = std::hint::black_box(ScalarIndicator::update(&mut g, x));
    });

    let mut rv = RollingVolatility::new(20);
    bench_scalar("RollingVolatility(20)", |x| {
        let _ = std::hint::black_box(ScalarIndicator::update(&mut rv, x));
    });

    let mut ph = PageHinkley::new(0.005, 1.0);
    bench_scalar("PageHinkley", |x| {
        let _ = std::hint::black_box(ph.update(x));
    });

    let mut tk = TeagerKaiser::new();
    bench_scalar("TeagerKaiser", |x| {
        let _ = std::hint::black_box(ScalarIndicator::update(&mut tk, x));
    });

    let mut kf = ConstantVelocityKalman::new(1.0, 1e-3, 1.0, 1.0);
    bench_scalar("KalmanFilter(CV)", |x| {
        let _ = std::hint::black_box(kf.update(x));
    });

    let mut lms = LmsFilter::new(8, 0.01);
    bench_scalar("LmsFilter(8)", |x| {
        let _ = std::hint::black_box(ScalarIndicator::update(&mut lms, x));
    });

    // Tier C: Welch PSD cadence cost. Measure the recompute cost specifically.
    let mut welch = WelchPsd::new(512, 512);
    let data: Vec<f64> = (0..(512 * 200)).map(|i| (i as f64 * 0.01).sin()).collect();
    // Fill once.
    for &x in data.iter().take(512) {
        welch.update(x);
    }
    let recomputes = 200;
    let t0 = Instant::now();
    let mut count = 0usize;
    for &x in data.iter() {
        if welch.update(x).is_some() {
            count += 1;
        }
    }
    let elapsed = t0.elapsed().as_nanos() as f64;
    println!(
        "{:<24} recompute mean={:8.0} ns  ({} recomputes, {} pushes)",
        "WelchPsd(512)",
        elapsed / count.max(1) as f64,
        count,
        data.len()
    );
    let _ = recomputes;

    // EwmaVolatility is constructed fresh in the closure above (measures
    // construction + update); the stateful ones below are the steady-state cost.
    let mut ewma = EwmaVolatility::risk_metrics();
    bench_scalar("EwmaVolatility(state)", |x| {
        let _ = std::hint::black_box(ScalarIndicator::update(&mut ewma, x));
    });
}
