# Task T-02478 Evidence: Sandbox Observability Hardening

## Goal
Harden the Sandbox Enforcement Observability subsystem against failures, resource exhaustion, unbounded collection, and malformed inputs.

## Hardening Mechanisms Enforced
1. **Query & Collection Size Caps**:
   - Query window bounded to `tail(1000)` records.
   - Outcome distribution maps capped at `MAX_OUTCOME_DISTRIBUTION_ENTRIES` (128 keys).
   - Profile distribution maps capped at `MAX_OUTCOME_DISTRIBUTION_ENTRIES` (128 keys).
2. **Text Sanitization & Invariant Guards**:
   - Telemetry strings are cleaned with `sanitize_telemetry_text`.
   - Control characters stripped (< 0x20 and 0x7F).
   - Length capped at `MAX_TELEMETRY_TEXT_LEN` (256 bytes).
3. **Graceful Degradation / Failure Envelope**:
   - If audit ring database connection is missing or uninitialized, report generation falls back gracefully without panics.
   - Any validation error returns an explicit standard error result code `SANDBOXOBS_ERR_VALIDATION`.
4. **Resource Management & Zero Leaks**:
   - No persistent file descriptors or locks held beyond ephemeral query tailing.
   - Temp files cleaned up after execution tests.

## Test Verification
- Ran `cargo test -p aiosh-core --test test_sandbox_observability`.
- Verified 5/5 unit tests pass, explicitly covering empty timestamps, cardinality bounds exceeding limits, and control character sanitization.
