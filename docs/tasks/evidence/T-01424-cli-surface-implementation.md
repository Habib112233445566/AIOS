# T-01424: User Session Bootstrap - CLI Surface: Implementation

## Metadata
- **Task ID:** `T-01424`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap CLI Surface Implementation (`code/aiosh-rust/aiosh-cli::cmd_session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (4/10) — CLI Surface Implementation

---

## 1. Implementation Deliverables

### 1.1 Complete `create` Subcommand
Implemented the full `create` subcommand in `code/aiosh-rust/aiosh-cli/src/main.rs`:
- **Input Processing & Size Bounds**:
  - Handles both file paths and inline JSON specification strings.
  - Enforces the 1 MiB ceiling (`PAYLOAD_TOO_LARGE`).
  - Employs strict error reporting on file read errors (`FILE_READ_ERROR`).
- **Schema & Invariant Validation**:
  - Deserializes into `UserSessionSpec` (`JSON_PARSE_ERROR`).
  - Executes `aiosh_core::session::validate_user_session_spec(&spec)` prior to store mutation (`VALIDATION_FAILED`).
- **Store Mutation & Atomic Persistence**:
  - Delegates to `UserSessionService::create_session(spec)`.
  - Atomically persists to disk using `service.save_to_path(p)` when `--store <path>` is supplied.
- **Structured Audit Logging (ADR-0035 §F-2)**:
  - Invokes `classify_and_emit` to capture non-repudiation audit records in the SQLite WAL ring buffer (`audit.db`) on every execution path.

### 1.2 Convenience Shortcuts & Aliases
Implemented first-class ergonomic shortcuts:
- `aiosh session status <id>` $\to$ maps directly to `show <id>`.
- `aiosh session activate <id>` $\to$ maps directly to `action <id> activate`.
- `aiosh session lock <id>` $\to$ maps directly to `action <id> lock`.
- `aiosh session unlock <id>` $\to$ maps directly to `action <id> unlock`.
- `aiosh session terminate <id>` $\to$ maps directly to `action <id> terminate`.
- `aiosh session auth <id>` $\to$ maps directly to `action <id> authenticate`.

---

## 2. Test Verification Output

### 2.1 CLI Smoke Test Suite (`python code/aiosh-cli/tests/test_session_cli_smoke.py`)
```text
=== RUNNING USER SESSION BOOTSTRAP CLI SMOKE TESTS ===
PASS: aiosh session --help
PASS: aiosh session unknown_cmd returns 2
PASS: aiosh session validate (id, user, spec, and json)
PASS: aiosh session list, show, action
PASS: aiosh session commands (status, shortcuts, create)
PASS: aiosh session hardening (payload limits, json parse, missing args, action envelopes)

ALL USER SESSION BOOTSTRAP CLI SMOKE TESTS PASSED!
```

### 2.2 Master Subsystem Matrix (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

---

## 3. Acceptance Verification
- [x] Targeted test passes (creation via inline JSON and file, status and action shortcuts).
- [x] No regressions across all existing session suites (`SB1..SB4`).
- [x] Full audit row emission verified across all success and error paths.
