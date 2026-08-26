# T-00156 — CI Smoke Orchestration / configuration: Integration

**Date:** 2026-08-25
**Feature:** CI Smoke Orchestration configuration

## 1. System Integration
- Wired `CiConfig.from_env()` into `tools/ci_suites.py`. The suite registry now scales `DEFAULT_TIMEOUT_S` and `RESULTS_PATH` directly from the validated configuration rather than static assignments. `RUST_SMOKE_TIMEOUT_S` is derived dynamically as `2 * timeout_default_s`.
- Updated `tools/ci_run.py` to use `_cfg.max_file_bytes` as the max buffer read boundary for log failure tails.
- Updated `code/aiosh-rust/aiosh-core/src/ci.rs` to invoke `CiConfig::from_env()?` at the top of `load_summary_with_retry`. The parsing size boundary (`cfg.max_file_bytes`), the retry ceiling (`cfg.load_retries`), and the poll interval (`cfg.retry_sleep_ms`) are now completely dynamically bound at runtime instead of hardcoded. 

## 2. Parity & Cross-Substrate Contract
- Both Python and Rust execution substrates derive their bounding limits from the identical environment contract, matching the behavior of the `aiosh task` domain and fulfilling the Twelve-Factor configuration specifications set out in T-00152.
