# T-01426: User Session Bootstrap - CLI Surface: Integration

## Metadata
- **Task ID:** `T-01426`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap CLI Surface Integration (`code/aiosh-rust/aiosh-cli::cmd_session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (6/10) — CLI Surface Integration

---

## 1. Production Surfaces Integrated

### 1.1 Root Command Dispatcher (`code/aiosh-rust/aiosh-cli/src/main.rs`)
- Top-level routing:
  ```rust
  Some("session") => cmd_session(&args[1..]),
  ```
- Updated top-level CLI usage string:
  ```text
  aiosh session <validate|list|show|status|create|action|activate|lock|unlock|terminate>  User Session Bootstrap Control
  ```

### 1.2 Subcommands & Lifecycle Shortcuts
1. **`validate`**:
   - `aiosh session validate (--id <id> | --user <user> | --spec <file_or_json>) [--json]`
   - Validates session IDs against `SB1`, usernames against `SB2`, or full specs against `SB1..SB5`.
2. **`list`**:
   - `aiosh session list [--user <user>] [--state <state>] [--type <type>] [--seat <seat>] [--limit <n>] [--store <path>] [--json]`
   - Formatted tabular terminal output or JSON array with strict `--limit` bounds validation ($1 \le n \le 10,000$).
3. **`show` / `status`**:
   - `aiosh session show <id> [--store <path>] [--json]` and `aiosh session status <id> [--store <path>] [--json]`.
   - Comprehensive inspection of session specification and runtime state.
4. **`action` & Direct Shortcuts**:
   - `aiosh session action <id> <authenticate|activate|lock|unlock|terminate> [--store <path>] [--json]`.
   - Ergonomic shortcut verbs forwarding directly with option preservation:
     - `aiosh session activate <id>`
     - `aiosh session lock <id>`
     - `aiosh session unlock <id>`
     - `aiosh session terminate <id>`
     - `aiosh session auth <id>`
5. **`create`**:
   - `aiosh session create <spec_file_or_json> [--store <path>] [--json]`.
   - Registers new user and agent sessions from file or inline JSON with 1 MiB payload bounds, schema validation, and capacity limit checks.

---

## 2. Cross-Substrate Parity & Audit

- **Standard Result Envelopes**:
  - In `--json` mode, all subcommands return standard envelopes:
    ```json
    {
      "code": 0,
      "data": { ... },
      "error": null
    }
    ```
    Matching the MCP JSON-RPC protocol and API expectations.
- **Synchronous Audit Logging (ADR-0035 §F-2)**:
  - Every invocation branch (both success and failure) calls `classify_and_emit`, capturing actor (`operator`), action, target, parameters, and outcome into `audit.log` and SQLite WAL ring (`audit.db`).
- **Atomic Persistence**:
  - Operations modifying state (`create`, `action`, shortcuts) serialize via `UserSessionService::save_to_path`, writing to process-isolated temp files (`.tmp.<pid>.<nanos>`) with atomic rename and clean unlinking on error.

---

## 3. Automated Test Verification

### 3.1 Rust CLI In-Tree Unit Test (`cargo test -p aiosh-cli --bin aiosh test_cmd_session_flow`)
```text
running 1 test
test task_cli_tests::test_cmd_session_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.71s
```

### 3.2 CLI Smoke Suite (`python code/aiosh-cli/tests/test_session_cli_smoke.py`)
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

### 3.3 Master Subsystem Matrix (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

---

## 4. Acceptance Verification
- [x] Feature is reachable through its production CLI surface (`aiosh session <subcommand>`).
- [x] Integration smoke and unit test suites pass end-to-end.
