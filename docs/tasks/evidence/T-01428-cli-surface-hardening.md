# T-01428: User Session Bootstrap - CLI Surface: Hardening

## Metadata
- **Task ID:** `T-01428`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap CLI Surface Hardening (`code/aiosh-rust/aiosh-cli::cmd_session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (8/10) — CLI Surface Hardening

---

## 1. Hardening Defenses & Invariants

### 1.1 Sizing Ceilings & Bounded Parameters
- **1 MiB Payload Cap**:
  - Enforced on `--spec` files, inline JSON, and `aiosh session create <spec_file_or_json>`.
  - Payloads exceeding `1,048,576` bytes are rejected prior to reading or parsing with exit code 2 and error code `PAYLOAD_TOO_LARGE`.
- **1,024 Character Store Path Bound**:
  - `--store <path>` is checked for maximum length ($\le 1,024$ bytes) and rejection of ASCII control characters (`c.is_control()`), returning exit code 2 (`INVALID_ARGUMENT`).
- **Bounded Query Limits**:
  - `--limit <n>` is constrained to strictly positive integers between $1$ and $10,000$, returning exit code 2 (`INVALID_ARGUMENT`) on invalid, zero, or negative bounds.
- **Session ID & Username String Length Bounds**:
  - Session IDs bounded to $[1 \dots 64]$ chars, usernames bounded to $[1 \dots 32]$ chars.

### 1.2 Standardized Result Envelopes (Zero Silent Failure)
- In `--json` mode, every failure mode produces a deterministic, machine-readable envelope:
  ```json
  {
    "code": 2,
    "data": null,
    "error": {
      "code": "MISSING_ARGUMENTS" | "INVALID_ARGUMENT" | "PAYLOAD_TOO_LARGE" | "JSON_PARSE_ERROR" | "VALIDATION_FAILED" | "UNKNOWN_SUBCOMMAND" | "ACTION_FAILED" | "CREATE_FAILED" | "FILE_READ_ERROR" | "LOAD_STORE_FAILED",
      "message": "..."
    }
  }
  ```
- In human/text mode, all errors output to stderr via `eprintln!` and return standard POSIX exit codes (1 for operational failures, 2 for invocation/syntax errors).

### 1.3 Resource Cleanup & Zero-Leak File Safety
- **Scoped File Reads**:
  - Input spec files are read within constrained lexical scopes and file handles dropped immediately after ingestion.
- **Process-Isolated Atomic Persistence**:
  - State mutations (`create`, `action`, shortcuts) persist using atomic temporary files (`<path>.tmp.<pid>.<nanos>`).
  - Temporary files are unlinked on any write, permission, or rename error path, ensuring zero leaked `.tmp` files.
- **Database Context Handles**:
  - SQLite WAL database context handles are cleaned up via RAII upon function return.

### 1.4 Fail-Open and Honest Audit Logging (ADR-0035 §F-2)
- Every subcommand execution branch (including invalid arguments, payload limit breaches, unknown commands, and illegal state transitions) emits a non-repudiation audit record via `classify_and_emit` into SQLite WAL ring buffer (`audit.db`) with SHA-256 hash chaining.

---

## 2. Test Verification Output

### 2.1 CLI Surface Smoke Suite (`python code/aiosh-cli/tests/test_session_cli_smoke.py`)
```text
=== RUNNING USER SESSION BOOTSTRAP CLI SMOKE TESTS ===
PASS: aiosh session --help
PASS: aiosh session unknown_cmd returns 2
PASS: aiosh session validate (id, user, spec, and json)
PASS: aiosh session list, show, action
PASS: aiosh session commands (status, shortcuts, create)
PASS: aiosh session hardening (payload limits, json parse, missing args, action envelopes, store & limit checks)

ALL USER SESSION BOOTSTRAP CLI SMOKE TESTS PASSED!
```

### 2.2 In-Tree Rust CLI Unit Test (`cargo test -p aiosh-cli --bin aiosh test_cmd_session_flow`)
```text
running 1 test
test task_cli_tests::test_cmd_session_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.71s
```

### 2.3 Master Subsystem Matrix (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

---

## 3. Acceptance Verification
- [x] Failure modes produce explicit, auditable errors across all subcommands.
- [x] No temp/connection leaks on error paths.
- [x] All hardening assertions verified in both standalone and master test runners.
