# Task Evidence: T-01976 - System Update / observability: Integration (Sub-Epic 8)

## Overview
- **Task ID**: `T-01976`
- **Component**: `aiosh-core::system_update_observability` & `aiosh-mcp`
- **Objective**: Conduct integration and smoke testing of the system update observability subsystem across Rust and Python MCP substrates to guarantee telemetry schema parity, invariant enforcement (`UOBS1..UOBS6`), and canonical JSON roundtrip fidelity.

## Test Execution Summary
The integration test suite `code/aiosh-mcp/tests/test_system_update_observability_smoke.py` was executed and all 6 integration test cases passed cleanly:
1. **Baseline Report Generation**: Validated clean idle service report structure, verifying all 22 fields are populated with correct defaults (slot A active, healthy, 0% progress).
2. **Text Sanitization & Clamping (`UOBS4`)**: Confirmed ASCII control characters are stripped, whitespace trimmed, and strings clamped to 256 characters across `last_error`, `target_version`, and `generated_at`. Progress $>100\%$ clamped to $100\%$.
3. **Staged Artifacts & Manifest Metrics (`UOBS2`)**: Verified accurate aggregation of staged artifact counts, payload bytes, and active manifest metadata (`update_id`, `channel`, `manifest_total_bytes`).
4. **Security Policy Integration (`UOBS3`)**: Verified policy evaluation synthesis (verdict, violation counts, policy mode) without side-effects.
5. **Health Status Evaluation (`UOBS6`)**: Verified system health computation across active slot success/failure and update failed state.
6. **Canonical JSON Serialization**: Validated lossless serialization and deserialization across the full 22-field schema.

## Status
- **Result**: PASS (100% test pass rate)
- **Sub-Epic 8 Status**: Progressing towards formal closure.
