# T-01438: User Session Bootstrap - MCP/API Surface: Hardening

## Metadata
- **Task ID:** `T-01438`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap MCP/API Surface Hardening (`code/aiosh-rust/aiosh-mcp`, `code/aiosh-rust/aiosh-core`, `code/aiosh-mcp/tests/test_session_mcp_smoke.py`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (8/10) — MCP/API Surface Hardening

---

## 1. Hardening Overview & Defenses Implemented

The MCP/API surface of User Session Bootstrap was subjected to rigorous defense-in-depth hardening across all 5 JSON-RPC tool endpoints:

### 1.1 Transport & Payload Size Ceilings
- **1 MiB Request Line Ceiling**: Enforced at the transport layer (`-32700: request line exceeds 1048576 bytes`).
- **1 MiB Ingestion Capping**: Explicitly enforced in `aios.session.validate` and `aios.session.create` for inline JSON objects and strings. Payloads exceeding $1,048,576$ bytes are rejected with explicit error strings before deserialization.
- **Environment Size Bounds (`SB4`)**: Bounded to at most 256 entries, key length $\le 128$ bytes, value length $\le 4,096$ bytes, and strict identifier syntax `^[A-Za-z_][A-Za-z0-9_]*$`.

### 1.2 Query Bounds & Resource Limits
- **Bounded Query Limits**: `aios.session.list` enforces $[1 \dots 10,000]$ bounds on the `limit` parameter. A value of $0$ or $>10,000$ returns an explicit error (`"Limit must be between 1 and 10,000"`).
- **Session Capacity Ceilings (`SB5`)**:
  - Maximum 32 active sessions per individual user account.
  - Maximum 1,024 total tracked sessions in the session store.

### 1.3 Path Sanitization & Injection Prevention
- **Identifier Validation (`SB1`)**: Applied on `aios.session.validate`, `aios.session.get`, and `aios.session.action`. Any session ID containing slashes, backslashes, null bytes, directory traversal patterns (`..`), or exceeding 64 characters is rejected prior to store lookup.
- **`store_path` Boundary Validation**: Uniformly enforced across all tools (`list`, `get`, `action`, `create`):
  - Path length bounded to $\le 1024$ characters.
  - Rejection of paths containing ASCII/Unicode control characters (`c.is_control()`).

### 1.4 Atomic Persistence & Temp Resource Cleanup
- **Atomic Two-Phase Writes**: Persistence via `UserSessionStore::save_to_path` writes to an isolated temporary file (`<path>.tmp.<pid>.<nanos>`) and atomically renames it over the destination file.
- **Clean Failure Recovery**: If serialization, directory creation, or file renaming fails, any created temporary file is unlinked immediately, preventing orphaned temporary files.

### 1.5 Explicit Error Envelopes & Audit Integrity (ADR-0035 §F-2)
- **Zero Silent Failures**: All error paths return a structured result envelope with `ok: false`, an explicit diagnostic error message, and a monotonic `audit_id`.
- **Honest Audit Logging**: Any error or failed validation path invokes `classify_and_emit()`, logging an honest failure row to SQLite WAL (`audit.db`) with `status: "failure"`, caller arguments, and error details.

---

## 2. Hardening Test Suite (`test_session_mcp_hardening`)

Added dedicated test function `test_session_mcp_hardening()` to `code/aiosh-mcp/tests/test_session_mcp_smoke.py`:
1. `aios.session.validate`: Oversized payload (>1 MiB) rejected with 1048576 bytes / 1 MiB message.
2. `aios.session.list`: `limit: 0` rejected with `"Limit must be between 1 and 10,000"`.
3. `aios.session.list`: `limit: 50000` rejected with `"Limit must be between 1 and 10,000"`.
4. `aios.session.list`: `store_path` > 1024 characters rejected.
5. `aios.session.list`: `store_path` with control characters rejected.
6. `aios.session.get`: Invalid `session_id` (`bad/id/traversal`) rejected with `"Invalid session_id"`.
7. `aios.session.get`: `store_path` > 1024 characters rejected.
8. `aios.session.action`: Traversal `session_id` (`../evil`) rejected with `"Invalid session_id"`.
9. `aios.session.action`: `store_path` > 1024 characters rejected.
10. `aios.session.create`: Oversized spec payload (>1 MiB) rejected.
11. `aios.session.create`: `store_path` > 1024 characters rejected.

---

## 3. Test Execution Outputs

### 3.1 Standalone Python MCP Smoke Suite
```text
=== RUNNING USER SESSION BOOTSTRAP MCP SMOKE TESTS ===
PASS: tools/list contains all 5 aios.session.* tools
PASS: aios.session.validate (valid, invalid, boundary, missing)
PASS: aios.session.list & aios.session.get (filtering, inspection, error modes)
PASS: aios.session.action (lock, unlock, unknown action, missing params)
PASS: aios.session.create & persistence (create, get, duplicate rejection, invalid spec)
PASS: Cross-surface CLI <-> MCP parity & state sharing
PASS: MCP session hardening (payload limits, query bounds, ID injection, store path sanitization)

ALL USER SESSION BOOTSTRAP MCP SMOKE TESTS PASSED!
```

### 3.2 In-Tree Cargo Test Suite (`cargo test -p aiosh-mcp`)
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.56s
     Running unittests src\main.rs (code\aiosh-rust\target\debug\deps\aiosh_mcp-570b1a936abd2622.exe)

running 11 tests
test tests::test_mcp_image_tools ... ok
test tests::test_mcp_handoff_tools ... ok
test tests::test_mcp_package_tools ... ok
test tests::test_mcp_distro_tools ... ok
test tests::test_mcp_doc_tools_execution ... ok
test tests::test_mcp_service_tools ... ok
test tests::test_mcp_session_validate_tools ... ok
test tests::test_toolchain_tools_in_manifest ... ok
test tests::test_mcp_triage_tools ... ok
test tests::test_mcp_repo_health_execution ... ok
test tests::test_mcp_secrets_tools_execution ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.90s
```

### 3.3 Master Session Subsystem Suite (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP in-tree unit test suite
[+] SB3 session MCP tool surface JSON-RPC smoke test (aios.session.*)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

---

## 4. Acceptance Verification

- [x] **Explicit failure envelopes**: All error modes return standard JSON-RPC envelopes with diagnostic error strings and audit IDs.
- [x] **No temp/connection leaks**: Atomic writes ensure immediate file cleanup on error.
- [x] **Bounded inputs**: Size caps, query limits, identifier checks, and path length restrictions enforced across all 5 tools.
- [x] **Honest audit trail**: All failure paths write failure rows to SQLite WAL.
