# T-00126 — CI Smoke Orchestration / core service: Integration

**Goal:** Integrate the core service of CI Smoke Orchestration with the surrounding system.

## Changes Made
1. **Service Hook**: Added `python3 tools/ci_service.py check` directly into `ci/run_all_smokes.sh`. This ensures that after `ci_run.py` generates the `/tmp/aiosh-ci-results.json` artifact, the result is immediately validated by the strict JSON read-only parser, acting as a final CI gate.
2. **CLI Registration Point**: Added an `aiosh ci <show|failures|check>` subcommand to the canonical Rust `aiosh-cli`. This delegates execution to the Python-based `ci_service.py` (respecting the "no MCP surface" and "read-only" constraints), making the interface highly discoverable for operators.
3. **Smoke Suite Parity**: Validated that `tools/test_ci_service.py` is included in the `SUITES` array within `tools/ci_suites.py` as `ci_service_unit`, meaning it is actively exercised during every CI run.

## Validation
- `cargo build` executed successfully without issues, verifying the new `aiosh ci` subcommand integration.
- `bash ci/run_all_smokes.sh` (or `python tools/ci_run.py` + `python tools/ci_service.py check`) natively executes all suites and reports `ci-check: PASS` as designed.

**Status:** Completed.
