# T-01416: User Session Bootstrap - Core Service: Integration

## Metadata
- **Task ID:** `T-01416`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Core Service Integration (`aiosh-cli`, `aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (6/10) — Core Service Integration

---

## 1. Production Surfaces Integrated

The `UserSessionService` core engine (`code/aiosh-rust/aiosh-core/src/session_service.rs`) has been wired into both production surfaces:

### 1.1 Operator CLI Surface (`code/aiosh-rust/aiosh-cli/src/main.rs`)
- **`aiosh session list`**:
  - Arguments: `[--user <username>] [--state <state>] [--type <type>] [--seat <seat>] [--limit <n>] [--json] [--store <path>]`
  - Dispatches multi-attribute search across tracked sessions in `UserSessionStore`.
  - Supports human-readable tabular output and JSON envelopes.
- **`aiosh session show <session_id>`** (alias `get`):
  - Arguments: `[--json] [--store <path>]`
  - Displays runtime session status (state, scope, leader PID, idle seconds, lock status, timestamps) and specification (user, UID, GID, type, class, seat, VTNR, display).
  - Returns exit code 0 if found, exit code 1 if not found (`NOT_FOUND`), exit code 2 if missing argument.
- **`aiosh session action <session_id> <action>`**:
  - Arguments: `authenticate`, `activate`, `lock`, `unlock`, `terminate` with `[--json] [--store <path>]`
  - Applies state machine transitions (`CS1`), seat arbitration (`CS2`), and lock consistency (`CS5`).
  - Emits classification audit events and atomically persists state updates to disk when `--store` is specified.
- **`aiosh session validate`**:
  - Validates session identifiers (SB1), usernames (SB2), and full specs (SB1..SB5).

### 1.2 Autonomous Agent MCP Tool Surface (`code/aiosh-rust/aiosh-mcp/src/main.rs`)
Registered in tool manifest and dispatched via `dispatch::recorded_call` under PEP authorization with SQLite WAL audit trails:
- **`aios.session.list`**: Filtered session inventory discovery.
- **`aios.session.get`**: Targeted inspection of status and spec.
- **`aios.session.action`**: Policy-enforced execution of lifecycle actions (`authenticate`, `activate`, `lock`, `unlock`, `terminate`).
- **`aios.session.validate`**: Syntax and schema verification.

---

## 2. Test Verification Outputs

### 1. CLI Smoke Suite (`python code/aiosh-cli/tests/test_session_cli_smoke.py`)
```text
=== RUNNING USER SESSION BOOTSTRAP CLI SMOKE TESTS ===
PASS: aiosh session --help
PASS: aiosh session unknown_cmd returns 2
PASS: aiosh session validate (id, user, spec, and json)
PASS: aiosh session list, show, action
PASS: aiosh session hardening (payload limits, json parse, missing args)

ALL USER SESSION BOOTSTRAP CLI SMOKE TESTS PASSED!
```

### 2. MCP Integration Suite (`cargo test -p aiosh-mcp --bin aiosh-mcp test_mcp_session_validate_tools`)
```text
running 1 test
test tests::test_mcp_session_validate_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.06s
```

### 3. Master Session Runner (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

---

## 3. Acceptance Verification

- [x] Feature reachable through production CLI commands (`aiosh session list`, `show`, `action`).
- [x] Feature reachable through autonomous agent MCP tools (`aios.session.list`, `get`, `action`).
- [x] Cross-substrate audit emission and atomic JSON persistence confirmed.
- [x] Integration smoke tests pass end-to-end.
