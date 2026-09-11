# T-01418: User Session Bootstrap - Core Service: Hardening

## Metadata
- **Task ID:** `T-01418`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Core Service Hardening (`code/aiosh-rust/aiosh-core::session_service`, `session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (8/10) — Core Service Hardening

---

## 1. Hardening Defenses & Invariants

### 1.1 Sizing Caps and Memory Bounds
- **10 MiB Store Size Cap (`MAX_SESSION_STORE_SIZE = 10,485,760` bytes)**:
  - Enforced prior to reading files into memory in `UserSessionStore::load_from_path` and `UserSessionService::load_from_path`.
  - Enforced in string deserialization (`UserSessionStore::from_json`).
- **Store Session Count Bounds (`MAX_TOTAL_SESSIONS = 1,024`)**:
  - `create_session` rejects insertions beyond 1,024 tracked sessions.
  - `from_json` rejects corrupted/adversarial payloads exceeding 1,024 entries upon deserialization.
- **Per-User Active Session Caps (`MAX_SESSIONS_PER_USER = 32`)**:
  - `create_session` rejects attempts by any user to exceed 32 concurrently active sessions (terminated sessions do not count against this active threshold).
- **1 MiB Specification Limit**:
  - CLI and MCP payload processing rejects inputs $> 1\text{ MiB}$ with `PAYLOAD_TOO_LARGE` before JSON parsing.

### 1.2 Defensive Atomic Persistence & Zero-Leak Resource Cleanup
- **Process-Isolated Tempfile Allocation**:
  - `save_to_path` isolates temporary files with unique process IDs and nanosecond timestamps (`<path>.tmp.<pid>.<nanos>`), avoiding collisions during concurrent session state updates.
- **Guaranteed Tempfile Cleanup on Error**:
  - Explicit cleanup handlers unlink and remove temporary files on write errors or rename failures, ensuring zero dangling `.tmp` files.
- **Bounded Retries on Atomic Rename**:
  - Renames execute with up to 3 bounded retries and exponential backoff (10ms) to accommodate transient filesystem or indexer locks.
- **Defensive Permissions**:
  - On Unix platforms, permissions are strictly configured to `0o600` (read/write restricted to root / supervisor only), safeguarding session state and credentials.

### 1.3 Standardized Result Envelopes (Never Silent Failure)
- Every failure mode emits an explicit, structured envelope:
  - Exit code `1`: Operational failures (`ACTION_FAILED`, `NOT_FOUND`, `LOAD_STORE_FAILED`).
  - Exit code `2`: Contract / syntax failures (`PAYLOAD_TOO_LARGE`, `JSON_PARSE_ERROR`, `MISSING_ARGUMENTS`, `INVALID_ACTION`, `VALIDATION_FAILED`).
  - JSON envelope schema:
    ```json
    {
      "code": 1,
      "data": null,
      "error": {
        "code": "ACTION_FAILED",
        "message": "Invalid session state transition: Active cannot execute Unlock"
      }
    }
    ```
- All public `UserSessionService` methods (`create_session`, `apply_action`, `update_idle`, `touch_activity`) return `Result<_, String>` with comprehensive diagnostic context.

### 1.4 Fail-Open and Non-Repudiation Audit Trail (ADR-0035 §F-2)
- All executions (both successful state mutations and negative rejection branches) emit a non-repudiation audit row into SQLite WAL ring buffer (`audit.db`) with SHA-256 hash chaining via `classify_and_emit`.

---

## 2. Test Verification Output

### 2.1 Core Service Unit & Hardening Suite (`cargo test --test test_session_service`)
```text
running 10 tests
test test_cs1_canonical_seeding_and_empty ... ok
test test_cs1_negative_transitions_and_error_handling ... ok
test test_cs1_session_lifecycle_state_machine ... ok
test test_cs2_seat_arbitration_and_foreground_uniqueness ... ok
test test_cs4_action_reporting_and_envelope_integrity ... ok
test test_cs5_idle_tracking_and_activity_resets ... ok
test test_cs3_capacity_limits_and_user_boundaries ... ok
test test_hardening_atomic_save_and_error_cleanup ... ok
test test_hardening_session_service_explicit_error_envelopes ... ok
test test_session_query_and_atomic_persistence ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
```

### 2.2 CLI Surface Smoke & Hardening (`python code/aiosh-cli/tests/test_session_cli_smoke.py`)
```text
=== RUNNING USER SESSION BOOTSTRAP CLI SMOKE TESTS ===
PASS: aiosh session --help
PASS: aiosh session unknown_cmd returns 2
PASS: aiosh session validate (id, user, spec, and json)
PASS: aiosh session list, show, action
PASS: aiosh session hardening (payload limits, json parse, missing args, action envelopes)

ALL USER SESSION BOOTSTRAP CLI SMOKE TESTS PASSED!
```

### 2.3 Master Subsystem Runner (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

---

## 3. Acceptance Verification
- [x] **Failure modes produce explicit, auditable errors**: Handled across payload ceilings, invalid syntax, missing args, unknown sessions, and illegal state transitions.
- [x] **No temp/connection leaks on the error path**: Verified via `test_hardening_atomic_save_and_error_cleanup` with isolated tempdir cleanup.
