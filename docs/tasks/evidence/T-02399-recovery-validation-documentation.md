# Task Evidence: T-02399 - Audit Chain Extensions: Recovery & Validation Documentation

## Goal
Document the recovery & validation subsystem of Audit Chain Extensions for operators, engineers, and automated agents.

## Documentation Summary
1. **Specification Update**:
   - Updated `docs/SPEC-AUDIT-EXTENSIONS.md` with Section 9 ("Audit Chain Recovery & Invariant Validation Subsystem").
   - Detailed structural diagnostics (`HashDiscontinuity`, `InvalidJson`, `CausalCycleDetected`, `SignatureMismatch`).
   - Detailed forward repair methodology (atomic snapshotting, non-destructive forward anchor events).
2. **Commands & Invocations**:
   - Documented CLI invocations: `aiosh audit validate [--json]` and `aiosh audit repair [--backup-dir <dir>] [--json]`.
   - Documented MCP JSON-RPC tool invocations: `aios.audit.validate` and `aios.audit.repair`.
3. **Constraints & Known Limitations**:
   - Directory traversal (`..`) forbidden in backup paths.
   - Max validation diagnostics collected clamped to 1,000 issues.
   - Historical rows are strictly immutable and never rewritten or deleted during repair.
4. **Evidence Cross-References**:
   - Linked tasks T-02391 to T-02400.
