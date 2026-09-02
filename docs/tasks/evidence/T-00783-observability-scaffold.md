# T-00783 — Secrets & Access Hygiene / observability: Scaffold

## 1. Observability Skeleton
- Added `severity_counts(&self) -> (u32, u32, u32, u32)` and `summary_line(&self) -> String` to `SecretScanReport` in `code/aiosh-rust/aiosh-core/src/secrets.rs`.
- Added unit test `test_secret_scan_report_observability` verifying severity counts breakdown and telemetry formatting.
- Verified build and unit test compilation.
