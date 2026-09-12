"""Python -> Rust boundary latency benchmark (design section 13).

Measures per-update cost as seen from Python, the value of grouped execution
via FeatureEngine, and scaling to many indicator instances.

Run:  python bench/latency.py
"""

import math
import time

import qstream as q


def pct(sorted_ns, p):
    idx = int(round((p / 100.0) * (len(sorted_ns) - 1)))
    return sorted_ns[idx]


def time_updates(n, fn):
    """Return (mean_ns, p50, p95, p99, throughput) for n calls of fn(i)."""
    # Warm up.
    for i in range(1000):
        fn(i)
    samples = []
    batch = 1000
    t_total = time.perf_counter_ns()
    for start in range(0, n, batch):
        t0 = time.perf_counter_ns()
        for i in range(start, min(start + batch, n)):
            fn(i)
        samples.append((time.perf_counter_ns() - t0) / batch)
    total = time.perf_counter_ns() - t_total
    samples.sort()
    mean = total / n
    return mean, pct(samples, 50), pct(samples, 95), pct(samples, 99), 1e9 / mean


def report(label, stats):
    mean, p50, p95, p99, thr = stats
    print(
        f"{label:<34} mean={mean:8.1f} ns  p50={p50:8.1f}  "
        f"p95={p95:8.1f}  p99={p99:8.1f}  thr={thr:9.0f}/s"
    )


def main():
    n = 300_000
    data = [((i % 97) - 48) * 0.001 for i in range(n)]
    prices = [100.0 + math.sin(i * 0.01) for i in range(n)]

    print(f"qstream Python->Rust latency benchmark (n={n:,})")
    print("-" * 92)

    # Single scalar indicator through PyO3.
    ewma = q.EwmaVolatility(alpha=0.06)
    report("EwmaVolatility.update", time_updates(n, lambda i: ewma.update(data[i])))

    garch = q.Garch()
    report("Garch.update", time_updates(n, lambda i: garch.update(data[i])))

    ph = q.PageHinkley()
    report("PageHinkley.update", time_updates(n, lambda i: ph.update(prices[i])))

    kf = q.KalmanFilter()
    report("KalmanFilter.update", time_updates(n, lambda i: kf.update(prices[i])))

    # Heavy periodic (Tier C): most calls just push into the ring buffer.
    welch = q.WelchPsd(window=256, update_every=128)
    report("WelchPsd.update (cadence)", time_updates(n, lambda i: welch.update(prices[i])))

    print("-" * 92)

    # FeatureEngine: amortize the Python<->Rust crossing across many features.
    kinds = [
        "ewma_volatility", "garch", "realized_volatility", "rolling_volatility",
        "page_hinkley", "teager_kaiser", "kalman", "adaptive_kalman",
        "lms", "rls", "savgol", "kz", "wiener", "sharpe", "sortino",
    ]
    names = [f"f{i}" for i in range(len(kinds))]
    channels = ["return" if k in ("ewma_volatility", "garch", "realized_volatility",
                                  "rolling_volatility", "sharpe", "sortino") else "close"
                for k in kinds]
    engine = q.FeatureEngine(names=names, kinds=kinds, channels=channels)

    def engine_step(i):
        px = prices[i]
        engine.update(px, px + 0.5, px - 0.5, px, 1000.0)

    eng_stats = time_updates(n, engine_step)
    report(f"FeatureEngine.update ({len(kinds)} feats)", eng_stats)
    per_feature = eng_stats[0] / len(kinds)
    print(f"    -> amortized per-feature cost: {per_feature:.1f} ns")

    # Compare: same 15 indicators updated individually from Python.
    inds = [
        q.EwmaVolatility(), q.Garch(), q.RealizedVolatility(period=20),
        q.RollingVolatility(period=20), q.PageHinkley(), q.TeagerKaiser(),
        q.KalmanFilter(), q.AdaptiveKalman(), q.LmsFilter(order=4),
        q.RlsFilter(order=4), q.SavitzkyGolay(window=9, order=2),
        q.KolmogorovZurbenko(window=11, passes=3), q.WienerFilter(),
        q.SharpeRatio(period=30), q.SortinoRatio(period=30),
    ]

    def individual_step(i):
        r = data[i]
        px = prices[i]
        inds[0].update(r); inds[1].update(r); inds[2].update(r)
        inds[3].update(r); inds[4].update(px); inds[5].update(px)
        inds[6].update(px); inds[7].update(px); inds[8].update(px)
        inds[9].update(px); inds[10].update(px); inds[11].update(px)
        inds[12].update(px); inds[13].update(r); inds[14].update(r)

    indiv_stats = time_updates(n, individual_step)
    report("15 indicators individually", indiv_stats)
    speedup = indiv_stats[0] / eng_stats[0]
    print(f"    -> FeatureEngine speedup vs individual calls: {speedup:.2f}x")

    print("-" * 92)

    # Scaling: cost of constructing/holding many independent instances.
    for count in (1, 100, 1000):
        many = [q.EwmaVolatility(alpha=0.06) for _ in range(count)]
        rounds = max(1, 200_000 // count)

        def many_step(i, many=many, rounds=rounds):
            x = data[i % len(data)]
            for ind in many:
                ind.update(x)

        t0 = time.perf_counter_ns()
        for i in range(rounds):
            many_step(i)
        elapsed = time.perf_counter_ns() - t0
        per_update = elapsed / (rounds * count)
        print(f"{count:>5} instances: {per_update:8.1f} ns per indicator update "
              f"({rounds * count:,} updates)")


if __name__ == "__main__":
    main()
