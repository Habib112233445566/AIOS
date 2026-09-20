# Task Evidence: T-01946 - System Update / Configuration: Integration

- **Task**: `T-01946`
- **Sub-Epic**: `Sub-Epic 5: Configuration & Policy`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Integration Work
Integrated `SystemUpdateConfig` into the live execution path of `aiosh-mcp`:
- **MCP Service Routing**: Integrated `SystemUpdateConfig::from_env()` inside `resolve_update_service()` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
- **Validation Enforcement**: Added `cfg.validate()` into `resolve_update_service()`, ensuring that any environment variable override (`AIOSH_UPDATE_STATE_DIR`, `AIOSH_UPDATE_STAGING_DIR`, etc.) is validated against invariants `UCONF1..UCONF3` before service instantiation.
- **Integration Smoke Suite**: Authored and executed `code/aiosh-mcp/tests/test_system_update_config_smoke.py`:
  1. Verified that setting `AIOSH_UPDATE_STATE_DIR` correctly configures the service without needing explicit command arguments.
  2. Verified that path traversal (`..`) or control characters in `AIOSH_UPDATE_STATE_DIR` are rejected at startup with structured error envelopes.
  3. Verified cross-substrate JSON serialization parity between Python and Rust.

## Verification
- Executed `python code/aiosh-mcp/tests/test_system_update_config_smoke.py`:
  - `ALL SYSTEM UPDATE CONFIGURATION INTEGRATION CHECKS PASSED.`
- Executed `python -m pytest code/aiosh-mcp/tests/test_system_update_config_smoke.py`:
  - `1 passed in 0.25s` (100% pass).
