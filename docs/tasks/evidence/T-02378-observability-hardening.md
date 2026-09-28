# Task Evidence: T-02378 - Audit Chain Extensions: Observability Hardening

## Goal
Harden the observability subsystem of Audit Chain Extensions against resource exhaustion, unbounded queries, and failure modes.

## Hardening Implemented
1. **Outcome Cardinality Bounding**:
   - Capped SQL outcome collection query to `LIMIT 128` with `ORDER BY COUNT(*) DESC`.
   - Enforced `MAX_OUTCOME_DISTRIBUTION_ENTRIES = 128` in-memory to prevent arbitrary hash map growth from hostile high-cardinality payloads.
2. **Session and Trace Memory Bounding**:
   - Enforced `MAX_TRACKED_CARDINALITY_ITEMS = 100_000` on in-memory deduplication hash sets (`sessions_set`, `traces_set`) during log traversal.
3. **Bounded Sanitization**:
   - `sanitize_telemetry_text` strictly enforces `MAX_TELEMETRY_TEXT_LEN = 256` chars, stripping control characters and trimming whitespaces.
4. **Error Propagation**:
   - Enforces explicit error codes (`AUDITOBS_ERR_QUERY`, `AUDITOBS_ERR_IO`, `AUDITOBS_ERR_VALIDATION`) rather than silent swallowing.
   - Resource leaks (SQLite statements, handles) are avoided via automatic RAII cleanup.

## Verification
- Unit test suite (`tests/test_audit_chain_observability.rs`) passed 5/5.
- Workspace builds cleanly with zero compiler warnings.
