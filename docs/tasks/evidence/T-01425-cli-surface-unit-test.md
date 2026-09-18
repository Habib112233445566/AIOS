# T-01425: User Session Bootstrap - CLI Surface: Unit Test

## Metadata
- **Task ID:** `T-01425`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap CLI Surface Unit Test (`code/aiosh-rust/aiosh-cli::task_cli_tests::test_cmd_session_flow`, `code/aiosh-cli/tests/test_session_cli_smoke.py`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (5/10) — CLI Surface Unit Test

---

## 1. Test Suite Deliverables

### 1.1 In-Tree Rust CLI Unit Test Suite (`test_cmd_session_flow`)
Added a dedicated comprehensive unit test in `code/aiosh-rust/aiosh-cli/src/main.rs` (`mod task_cli_tests::test_cmd_session_flow`) covering all CLI command branches and return codes:
- **Help & Unknown Subcommands**:
  - `aiosh session --help` $\to$ exit code 0.
  - `aiosh session unknown_cmd` $\to$ exit code 2.
- **Validation Operations**:
  - Valid session ID (`sess-01`) $\to$ exit code 0; invalid session ID (`../evil`) $\to$ exit code 2.
  - Valid username (`kali`) $\to$ exit code 0; invalid username (`Kali`) $\to$ exit code 2.
  - Valid spec $\to$ exit code 0; invalid spec (missing display for X11) $\to$ exit code 2.
- **Session Queries**:
  - Default `aiosh session list` and `aiosh session list --json` $\to$ exit code 0.
  - Invalid limit boundary (`--limit 0`) $\to$ exit code 2 (`INVALID_ARGUMENT`).
- **Inspection & Status**:
  - `aiosh session show greeter-seat0` and `--json` $\to$ exit code 0.
  - `aiosh session status greeter-seat0 --json` (status alias) $\to$ exit code 0.
  - Missing session ID $\to$ exit code 2; non-existent session $\to$ exit code 1.
- **Lifecycle Actions & Shortcuts**:
  - `aiosh session action greeter-seat0 lock` $\to$ exit code 0.
  - Unknown action $\to$ exit code 2.
  - Convenience shortcuts: `aiosh session lock greeter-seat0`, `aiosh session activate greeter-seat0` $\to$ exit code 0.
- **Session Creation**:
  - `aiosh session create <valid_spec> --json` $\to$ exit code 0.
  - `aiosh session create <invalid_spec> --json` $\to$ exit code 2.
  - `aiosh session create` (missing arguments) $\to$ exit code 2.

### 1.2 CLI Smoke Suite Extension (`test_session_cli_smoke.py`)
Extended `code/aiosh-cli/tests/test_session_cli_smoke.py` with standalone assertions:
- `test_session_commands`: Validates `status`, `lock`, `unlock` (with stateful `--store` persistence), `activate`, and `create` (valid inline spec vs invalid spec).

---

## 2. Test Verification Output

### 2.1 In-Tree Rust Unit Test (`cargo test -p aiosh-cli --bin aiosh test_cmd_session_flow`)
```text
   Compiling aiosh-cli v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.95s
     Running unittests src\main.rs (target\debug\deps\aiosh-8814d271a6af31ad.exe)

running 1 test
test task_cli_tests::test_cmd_session_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.71s
```

### 2.2 Standalone Python Smoke Test (`python code/aiosh-cli/tests/test_session_cli_smoke.py`)
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

### 2.3 Master Subsystem Suite (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

---

## 3. Acceptance Verification
- [x] Test suite runs standalone and passes cleanly.
- [x] Both valid and negative edge cases thoroughly asserted.
- [x] Zero regressions across touched components.
