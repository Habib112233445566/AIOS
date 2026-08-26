# T-00010 — Audit-ring performance baseline

Completed: 2026-08-21T06:38:06.744127+00:00

Acceptance criteria:
- [x] Benchmark table exists with a recommended max live-row threshold.

Note: Benchmarks: 10k rows verify 0.574s/rotate 1.327s/verify_full 0.344s; 100k rows verify 6.251s/rotate 25.9s/verify_full 6.02s; post-rotate verify ~0.05-0.15s (bounded by keep_rows, VACUUM irrelevant). Recommendation: rotate at 10k live rows, keep 1000.
