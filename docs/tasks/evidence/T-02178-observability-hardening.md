# Task Evidence: T-02178 - PEP Decision Engine: Observability: Hardening

## Task Metadata
- **Task ID**: `T-02178`
- **Sub-Epic**: Sub-Epic 8: Observability Subsystem
- **Component**: `aiosh-core::pep_observability`, `aiosh-cli`, `aiosh-mcp`
- **Date**: 2026-09-22
- **Status**: Completed

## 1. Hardening Measures Implemented

1. **Size Caps & Invariant Bounds**:
   - `MAX_PEPOBS_TEXT_LEN` (256 bytes) caps any telemetry text (timestamps, store paths).
   - `MAX_RULES_IN_SERVICE` (5,000 rules) limits inventory size.
   - Capacity utilization bounded to 0..=100% via `.min(100) as u8`.
   - `validate()` enforces cross-field sum invariants: `sum(rules_by_effect) == total_rules` and `rules_with_obligations <= total_rules`.

2. **Standard Result Envelopes (No Silent Failure)**:
   - CLI outputs standard envelope:
     ```json
     {
       "code": 0,
       "data": { ... },
       "error": null
     }
     ```
   - Validation failure outputs standard error envelope with non-zero exit code (1 for validation failure, 2 for syntax/path error):
     ```json
     {
       "code": 1,
       "data": null,
       "error": {
         "code": "VALIDATION_FAILED",
         "message": "PEPOBS_ERR_VALIDATION: ..."
       }
     }
     ```
   - MCP outputs `{ "ok": true, "tool": "aios.pep.report", "report": { ... } }` or `{ "ok": false, "error": ... }`.

3. **Resource Management**:
   - Zero temp files generated during report generation.
   - Pure, non-blocking in-memory aggregation.
   - Memory footprint is proportional to rule metadata counts, without cloning rule obligation payloads.

4. **Honest Audit Emission (ADR-0035 §F-2)**:
   - Any failure path in report generation (corrupted store, validation rejection) writes an honest audit row to the SQLite WAL ring before returning.

## 2. Verification
- All smoke suites (`test_pep_cli_smoke.py`, `test_pep_decision_smoke.py`) and unit tests (`test_pep_observability.rs`) verify that error paths produce auditable output and clean returns.
