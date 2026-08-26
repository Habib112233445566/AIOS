# T-00154 — CI Smoke Orchestration / configuration: Implementation

**Date:** 2026-08-25
**Feature:** CI Smoke Orchestration configuration

## 1. Minimal Working Behavior
- Fully implemented `code/aiosh-rust/aiosh-core/src/ci_config.rs` (`CiConfig::from_env` and `from_source`) and its python equivalent in `tools/ci_config.py`.
- Both implementations perform strict bounds checking on `AIOSH_CI_RESULTS`, `AIOSH_CI_MAX_FILE_BYTES`, `AIOSH_CI_TIMEOUT_DEFAULT_S`, `AIOSH_CI_LOAD_RETRIES`, and `AIOSH_CI_RETRY_SLEEP_MS`.
- The CLI command `aiosh ci config` is fully wired in `code/aiosh-rust/aiosh-cli/src/main.rs`, replacing the scaffolded `unimplemented!()` block. It resolves the environment config and prints the JSON object with source metadata (`env` vs `default`).

## 2. Test Passing
- Wrote and executed `tools/test_ci_config.py` (which passed). It verifies that default values are used when environment variables are unset, environment variables override defaults perfectly, and malformed inputs (non-integers, below-floor values) raise loud errors specifying exactly which variable failed.
