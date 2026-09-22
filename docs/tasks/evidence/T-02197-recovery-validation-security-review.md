# Task Evidence: T-02197 (recovery & validation: Security Review)

## 1. Scope & Execution
Conducted an exhaustive security audit of the PEP Decision Engine Recovery & Validation subsystem (`code/aiosh-rust/aiosh-core/src/pep_recovery.rs`, CLI and MCP surfaces):
- Checked path sanitization and directory traversal prevention.
- Checked deserialization defenses against memory exhaustion and large payload attacks.
- Checked rule validation semantics (null-byte, control character, length checks).
- Verified audit-row emission through the MCP dispatch recording gate.
- Documented 8 concrete abuse scenarios and test results in `docs/tasks/evidence/T-02197-security.md`.

## 2. Findings & Verification
- All input paths are validated against `..`, NUL bytes, and malformed characters via `validate_path_hygiene`.
- Store size capped at 10 MiB (`MAX_PEP_SERVICE_STORE_SIZE = 10 * 1024 * 1024`), verified before memory allocation.
- Rule capacity capped at 5,000 rules (`MAX_RULES_IN_SERVICE = 5000`), preventing algorithmic complexity attacks.
- Atomic swap ensures zero risk of partial file writes or corruption on crash during recovery.
- MCP invocations consistently route through `dispatch::recorded_call`, writing an immutable SQLite audit row for both successful and rejected operations.

## 3. Acceptance Confirmation
- [x] Security evidence file exists at `docs/tasks/evidence/T-02197-security.md` with abuse scenarios.
- [x] All 8 abuse scenarios verified mitigated.
- [x] No known policy bypass remains open.
