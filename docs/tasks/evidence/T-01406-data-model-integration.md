# T-01406: User Session Bootstrap - Data Model: Integration

## Metadata
- **Task ID:** `T-01406`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Data Model Integration (`code/aiosh-rust/aiosh-core::session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (6/10) — Data Model Integration

---

## 1. Integrated Surfaces

### Operator CLI (`aiosh session validate`)
- Integrated `cmd_session` in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - `aiosh session validate --id <session_id> [--json]`: Validates session identifier syntax against SB1 invariants (1..64 chars, alphanumeric, allowed symbols `[a-zA-Z0-9_.-]`, rejects traversal sequences `..`, slashes, whitespace, and metacharacters).
  - `aiosh session validate --user <username> [--json]`: Validates user account syntax against SB2 invariants (1..32 chars, lowercase/underscore start, POSIX compliance).
  - `aiosh session validate --spec <file_or_json> [--json]`: Validates complete `UserSessionSpec` against SB1..SB5 invariants (seats, VTNR range, display strings, environment key format, path isolation).
  - Emits non-repudiation audit rows to SQLite WAL ring buffer via `classify_and_emit` (`tool: "session"`, `command: "validate"`).
  - Structured output formatting: human-readable prose for terminal operators and standard JSON envelope with `code`, `data`, and `error` for automated systems.

### Autonomous Agent MCP Tool (`aios.session.validate`)
- Registered `aios.session.validate` in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
  - Input Schema: `session_id?: string`, `username?: string`, `spec?: object`, `grant_id?: string`.
  - Dispatched via `recorded_call` ensuring PEP authorization checks and immutable audit logging.
  - Returns standard response envelope with `valid: bool`, details, and any invariant violation error lists.

### Subsystem Master Test Runner Matrix (`tools/test_session_suites.py`)
- Created dedicated test runner `tools/test_session_suites.py`:
  - Criterion `SB1`: Session data model integrity & invariants (`test_session_data_model`).
  - Criterion `SB2`: Session CLI surface commands & options (`test_session_cli_smoke.py`).
  - Criterion `SB3`: Session MCP tool surface (`test_mcp_session_validate_tools`).

---

## 2. Test Verification Outputs

### 1. Master Session Subsystem Suite (`tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)

PASS: session_suites criteria (SB1..SB3)
```

### 2. CLI Smoke Test (`python code/aiosh-cli/tests/test_session_cli_smoke.py`)
```text
=== RUNNING USER SESSION BOOTSTRAP CLI SMOKE TESTS ===
PASS: aiosh session --help
PASS: aiosh session unknown_cmd returns 2
PASS: aiosh session validate (id, user, spec, and json)

ALL USER SESSION BOOTSTRAP CLI SMOKE TESTS PASSED!
```

### 3. MCP Tool Test (`cargo test -p aiosh-mcp --bin aiosh-mcp test_mcp_session_validate_tools`)
```text
running 1 test
test tests::test_mcp_session_validate_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 1.99s
```

---

## 3. Acceptance Verification
- [x] Feature reachable through its production surfaces (CLI `aiosh session validate` and MCP `aios.session.validate`).
- [x] Integration smoke passes end-to-end (`test_session_cli_smoke.py`, MCP test, `test_session_suites.py`).
