# T-01423: User Session Bootstrap - CLI Surface: Scaffold

## Metadata
- **Task ID:** `T-01423`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap CLI Surface Scaffold (`code/aiosh-rust/aiosh-cli::cmd_session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (3/10) — CLI Surface Scaffold

---

## 1. Scaffold Deliverables

### 1.1 Extended CLI Dispatcher (`cmd_session`)
Extended `cmd_session` in `code/aiosh-rust/aiosh-cli/src/main.rs` with typed dispatch branches and fail-loud stubs:
- `create`:
  - Syntax: `aiosh session create <spec_file_or_json> [--json] [--store <path>]`
  - Validates argument presence (returns exit code 2 on missing argument).
  - Scaffold stub: Fails loudly with exit code 1, emits audit record via `classify_and_emit`, and outputs standard error envelope `{"code": 1, "data": null, "error": {"code": "NOT_IMPLEMENTED", "message": "session create is scaffolded but not yet implemented: '<input>'"}}`.
- `status`:
  - Syntax: `aiosh session status <session_id> [--json] [--store <path>]`
  - Ergonomic alias directly delegating to `cmd_session(&["show", ...])`.
- Action Shortcuts:
  - `activate`: `aiosh session activate <session_id> [--json] [--store <path>]` (delegates to `action <id> activate`).
  - `lock`: `aiosh session lock <session_id> [--json] [--store <path>]` (delegates to `action <id> lock`).
  - `unlock`: `aiosh session unlock <session_id> [--json] [--store <path>]` (delegates to `action <id> unlock`).
  - `terminate`: `aiosh session terminate <session_id> [--json] [--store <path>]` (delegates to `action <id> terminate`).
  - `auth` / `authenticate`: `aiosh session auth <session_id> [--json] [--store <path>]` (delegates to `action <id> authenticate`).
- Help text updated in both top-level `aiosh --help` and `aiosh session --help` to document all supported commands.

### 1.2 Automated Test Stub Coverage
Added test function `test_session_scaffolded_commands` to `code/aiosh-cli/tests/test_session_cli_smoke.py`:
- Verified `status` alias retrieves session status.
- Verified `lock`, `unlock`, and `activate` action shortcuts correctly execute transitions with persistent `--store` integration.
- Verified `create` missing argument returns exit code 2 (`MISSING_ARGUMENTS`).
- Verified `create` scaffold stub fails loudly with exit code 1 (`NOT_IMPLEMENTED`).

---

## 2. Compilation & Verification Output

### 2.1 Cargo Build (`cargo build -p aiosh-cli`)
```text
   Compiling aiosh-core v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-core)
   Compiling aiosh-cli v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 26.17s
```

### 2.2 CLI Surface Smoke Suite (`python code/aiosh-cli/tests/test_session_cli_smoke.py`)
```text
=== RUNNING USER SESSION BOOTSTRAP CLI SMOKE TESTS ===
PASS: aiosh session --help
PASS: aiosh session unknown_cmd returns 2
PASS: aiosh session validate (id, user, spec, and json)
PASS: aiosh session list, show, action
PASS: aiosh session scaffolded commands (status, shortcuts, create stub)
PASS: aiosh session hardening (payload limits, json parse, missing args, action envelopes)

ALL USER SESSION BOOTSTRAP CLI SMOKE TESTS PASSED!
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
- [x] Project builds with zero errors or warnings.
- [x] New interfaces exist, fail loudly, and are verified by test stubs in `test_session_cli_smoke.py`.
