# T-01408: User Session Bootstrap - Data Model: Hardening

## Metadata
- **Task ID:** `T-01408`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Data Model Hardening (`code/aiosh-rust/aiosh-core::session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (8/10) — Data Model Hardening

---

## 1. Hardening Defenses & Invariants

### 1. Payload Sizing Limits & Memory Bounds
- **1 MiB Payload Cap**: Enforced a hard 1 MiB (`1,048,576` bytes) ceiling on specification files and inline JSON strings in CLI `aiosh session validate --spec <input>`. Oversized payloads are rejected with exit code 2 and structured error code `PAYLOAD_TOO_LARGE` prior to JSON parsing or memory expansion.
- **10 MiB Store Size Cap**: Enforced a 10 MiB (`MAX_SESSION_STORE_SIZE`) maximum payload ceiling in `UserSessionStore::from_json` and `load_from_path`.
- **In-Memory Type Bounding**:
  - `session_id`: $[1 \dots 64]$ bytes, regex `^[a-zA-Z0-9][a-zA-Z0-9_.-]{0,63}$`.
  - `username`: $[1 \dots 32]$ bytes, POSIX lowercase/underscore syntax.
  - `seat`: $[1 \dots 32]$ bytes, `seat` prefix.
  - `vtnr`: Range $[1 \dots 12]$.
  - `display`: Prefix `:`, max 16 bytes.
  - `environment`: Max 256 keys, keys max 64 chars, values max 4,096 chars, zero null bytes.
  - Capacity: Max 32 active sessions per user, max 1,024 total sessions in store.

### 2. Explicit Result Envelopes
- Never fails silently. Every failure branch produces a structured result:
  - Exit code `2` for validation errors, missing arguments, or invalid syntax.
  - Standard JSON envelope:
    ```json
    {
      "code": 2,
      "data": null,
      "error": {
        "code": "PAYLOAD_TOO_LARGE" | "JSON_PARSE_ERROR" | "FILE_READ_ERROR" | "MISSING_ARGUMENTS" | "VALIDATION_FAILED" | "UNKNOWN_SUBCOMMAND",
        "message": "...",
        "errors": [...]
      }
    }
    ```

### 3. Resource Cleanup & Leak Prevention
- Scoped file reads ensure file descriptors are dropped immediately after read.
- `UserSessionStore::save_to_path` employs atomic temporary file replacement (`.tmp` write followed by rename) to prevent torn states.
- Temporary files in test suites are cleanly removed via RAII or `finally` blocks.

### 4. Honest Audit Trail (ADR-0035 §F-2)
- All executions (both successful validations and negative rejections due to syntax, size limits, or missing arguments) emit a non-repudiation audit row into SQLite WAL ring buffer (`audit.db`) with SHA-256 hash chaining via `classify_and_emit`.

---

## 2. Test Verification Output (`python code/aiosh-cli/tests/test_session_cli_smoke.py`)
```text
=== RUNNING USER SESSION BOOTSTRAP CLI SMOKE TESTS ===
PASS: aiosh session --help
PASS: aiosh session unknown_cmd returns 2
PASS: aiosh session validate (id, user, spec, and json)
PASS: aiosh session hardening (payload limits, json parse, missing args)

ALL USER SESSION BOOTSTRAP CLI SMOKE TESTS PASSED!
```

### Master Subsystem Suite (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)

PASS: session_suites criteria (SB1..SB3)
```

---

## 3. Acceptance Verification
- [x] Failure modes produce explicit, auditable errors (all tested: payload limits, malformed JSON, missing args).
- [x] Zero resource leaks or dangling file handles on error paths.
