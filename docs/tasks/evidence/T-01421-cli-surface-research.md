# T-01421: User Session Bootstrap - CLI Surface: Research

## Metadata
- **Task ID:** `T-01421`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap CLI Surface Research (`code/aiosh-rust/aiosh-cli::cmd_session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (1/10) — CLI Surface Research

---

## 1. Objectives & Scope

Research the operator-facing command-line interface (`aiosh session`) for inspecting, querying, validating, managing lifecycle transitions, and registering sessions on AIOS. The CLI surface must fulfill:
1. **Interactive Usability**: Clean, aligned terminal representation for interactive operator sessions (similar to `loginctl(1)`).
2. **Machine Parsing**: Deterministic, structured JSON envelopes when `--json` is specified for scripts, agents, and downstream orchestrators.
3. **Auditability (ADR-0035 §F-2)**: Synchronous audit row emission via `classify_and_emit` on every CLI invocation branch (both success and failure).
4. **Defensive Sizing & Validation**: Strict 1 MiB payload ceiling on specification inputs, string length limits, control character rejection, and standard POSIX exit code contracts (0 = success, 1 = operational failure, 2 = syntax/argument error).

---

## 2. Prior Art & Authoritative Sources

- **`loginctl(1)` (systemd-logind control interface)**: Standard Linux login and session manager CLI.
  - Primary subcommands: `list-sessions`, `session-status <id>`, `show-session <id>`, `activate <id>`, `lock-session <id>`, `unlock-session <id>`, `terminate-session <id>`, `kill-session <id>`.
  - Columnar output: `SESSION`, `UID`, `USER`, `SEAT`, `TTY`, `STATE`.
- **`w(1)` / `who(1)` / `last(1)` (POSIX / Linux coreutils / utmp / wtmp)**: Traditional Unix session reporting showing active terminals, idle duration, and login timestamps.
- **`pam_systemd(8)` & `pam_open_session(3)`**: Authoritative specification for establishing PAM environments, allocating session scopes, and setting `$XDG_RUNTIME_DIR`.
- **POSIX.1-2017 Utility Conventions (IEEE Std 1003.1-2017)**: Guidelines 3 through 10 governing standard option syntax, argument validation, and return code semantics (0 for success, 1 for operational error, 2 for syntax/argument error).

---

## 3. Fact vs. Assumption Separation

### Established Facts
- **Fact 1 (Data Model & Core Service Foundation)**: `aiosh-core::session` and `aiosh-core::session_service` provide canonical data structures (`UserSessionSpec`, `UserSessionStatus`, `UserSessionQuery`, `UserSessionActionReport`, `UserSessionStore`, `UserSessionService`) enforcing invariants `SB1..SB5` and `CS1..CS5`.
- **Fact 2 (Existing Subcommands)**: `aiosh session` currently implements:
  - `validate (--id <id> | --user <user> | --spec <file_or_json>) [--json]`
  - `list [--user <username>] [--state <state>] [--type <type>] [--seat <seat>] [--limit <n>] [--store <path>] [--json]`
  - `show <session_id> [--store <path>] [--json]`
  - `action <session_id> <authenticate|activate|lock|unlock|terminate> [--store <path>] [--json]`
- **Fact 3 (Audit Emission)**: Every execution path calls `classify_and_emit` to record event details to the SQLite WAL audit ring or `audit.log` with non-repudiation SHA-256 hash chaining.
- **Fact 4 (Memory & Input Hardening)**: Input reads are strictly bounded: 1 MiB limit on specification files/strings, 1024 characters on `--store`, 64 characters on session IDs, 32 characters on usernames/seats, control characters rejected.
- **Fact 5 (Exit Code Contract)**: Return codes follow standard semantics: 0 = success, 1 = operational/store failure, 2 = argument/syntax error.

### Engineering Assumptions
- **Assumption 1 (Convenience Shortcuts)**: Operators familiar with `loginctl` will benefit from convenience shortcut verbs:
  - `status <id>` (alias for `show <id>`)
  - `activate <id>` (alias for `action <id> activate`)
  - `lock <id>` (alias for `action <id> lock`)
  - `unlock <id>` (alias for `action <id> unlock`)
  - `terminate <id>` (alias for `action <id> terminate`)
- **Assumption 2 (Session Creation Command)**: Adding a dedicated `create <spec_file_or_json> [--store <path>] [--json]` subcommand will allow system bootstrap scripts and agent initiators to register sessions without writing manual store files.
- **Assumption 3 (Sandbox/Store Isolation)**: The `--store <path>` flag allows operators to inspect or manipulate offline/chroot session stores without altering active running system state.

---

## 4. Proposed CLI Grammar & Command Taxonomy

```text
aiosh session <subcommand> [options]

Subcommands:
  validate   (--id <id> | --user <username> | --spec <file_or_json>) [--json]
  list       [--user <username>] [--state <state>] [--type <type>] [--seat <seat>] [--limit <n>] [--store <path>] [--json]
  show       <session_id> [--store <path>] [--json]
  status     <session_id> [--store <path>] [--json]          (alias for show)
  action     <session_id> <authenticate|activate|lock|unlock|terminate> [--store <path>] [--json]
  create     <spec_file_or_json> [--store <path>] [--json]
  activate   <session_id> [--store <path>] [--json]          (alias for action <id> activate)
  lock       <session_id> [--store <path>] [--json]          (alias for action <id> lock)
  unlock     <session_id> [--store <path>] [--json]          (alias for action <id> unlock)
  terminate  <session_id> [--store <path>] [--json]          (alias for action <id> terminate)

Flags:
  --json           Format response in canonical JSON envelope
  --store <path>   Explicit path to persistent session store file
  --help, -h       Display help and usage details
```

---

## 5. Exit Code Contract

- `0`: Success (validation passed, session created, action applied, session shown, list returned).
- `1`: Operational error (session not found, action rejected by state machine / seat arbitration, store load failure, persistence error).
- `2`: Syntax or invocation error (missing required arguments, unrecognized subcommand or action, payload $> 1\text{ MiB}$, invalid session ID or username syntax).

---

## 6. Decisions & Unknowns for Specification Phase (T-01422)

1. **Decision 1 (Ergonomic Aliases)**: Implement `status`, `activate`, `lock`, `unlock`, and `terminate` as first-class CLI shortcuts directly delegating to underlying `show` and `action` operations, retaining 100% backward compatibility.
2. **Decision 2 (Session Creation)**: Implement `create <spec_file_or_json>` using the hardened 1 MiB file/inline parser and `UserSessionService::create_session`.
3. **Decision 3 (Store Persistence on Action/Create)**: When `--store <path>` is provided, write back mutations using `service.save_to_path(p)` and fail with explicit error code `PERSIST_FAILED` if file writing fails.
4. **Decision 4 (Test Matrix Extension)**: Extend `code/aiosh-cli/tests/test_session_cli_smoke.py` and `tools/test_session_suites.py` to cover all new subcommands and aliases under criterion `SB2`.

---

## 7. Acceptance Verification
- [x] Evidence file exists and cleanly separates established facts from engineering assumptions.
- [x] No source code was modified during this research phase.
- [x] Decisions and unknowns needed for specification phase are explicitly listed.
