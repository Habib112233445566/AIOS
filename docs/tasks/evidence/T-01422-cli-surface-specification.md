# T-01422: User Session Bootstrap - CLI Surface: Specification

## Metadata
- **Task ID:** `T-01422`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap CLI Surface Specification (`code/aiosh-rust/aiosh-cli::cmd_session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (2/10) — CLI Surface Specification

---

## 1. CLI Dispatch Specification (`aiosh session`)

### 1.1 Command Interface & Entry Point
```rust
fn cmd_session(args: &[String]) -> i32
```
Dispatched from `code/aiosh-rust/aiosh-cli/src/main.rs`:
```rust
Some("session") => cmd_session(&args[1..]),
```

### 1.2 Subcommands & Functional Contracts

#### 1. `validate`
- **Syntax**: `aiosh session validate (--id <id> | --user <username> | --spec <file_or_json>) [--json]`
- **Inputs**: Session identifier string, username string, or serialized `UserSessionSpec` (file path or inline JSON string).
- **Validation**: Enforces 1 MiB payload ceiling. Validates syntax against `SB1` (ID bounds & regex), `SB2` (username POSIX rules), or full `SB1..SB5` invariants.
- **Output**:
  - Text: `VALID: ...` or `INVALID: ...` with detailed invariant error list.
  - JSON: Envelope `{"code": 0|2, "data": {"valid": bool, ...}, "error": ...}`.
- **Exit Codes**: `0` on valid, `2` on validation violation, payload $> 1\text{ MiB}$, or missing arguments.
- **Persistence**: Read-only; zero disk side effects.

#### 2. `list`
- **Syntax**: `aiosh session list [--user <username>] [--state <state>] [--type <type>] [--seat <seat>] [--limit <n>] [--store <path>] [--json]`
- **Inputs**: Filter flags (username, session state, session type, physical/virtual seat, integer limit) and optional store path.
- **Output**:
  - Text: Formatted table with `SESSION_ID`, `USER`, `STATE`, `SCOPE`, `LOCKED`, `IDLE`.
  - JSON: Result envelope `{"code": 0, "data": {"sessions": [...], "count": n}, "error": null}`.
- **Exit Codes**: `0` on success, `1` on store loading error, `2` on invalid argument.
- **Persistence**: Read-only.

#### 3. `show` / `status`
- **Syntax**: `aiosh session show <session_id> [--store <path>] [--json]`
- **Alias (AIOS-specific)**: `aiosh session status <session_id> [--store <path>] [--json]`
- **Inputs**: Target session identifier and optional store path.
- **Output**:
  - Text: Detailed multi-line attribute view (Session ID, Username, UID, State, Scope, Type, Class, Seat, VTNR, Display, Leader PID, Locked, Idle Seconds, Created At, Last Active At).
  - JSON: Result envelope `{"code": 0, "data": {"status": ..., "spec": ...}, "error": null}`.
- **Exit Codes**: `0` on found, `1` on not found or store read error, `2` on missing session ID.
- **Persistence**: Read-only.

#### 4. `action`
- **Syntax**: `aiosh session action <session_id> <authenticate|activate|lock|unlock|terminate> [--store <path>] [--json]`
- **Inputs**: Target session identifier, administrative lifecycle action, and optional store path.
- **Output**:
  - Text: Outcome summary displaying state transition (`Action 'activate' applied to session 'sess-01': Authenticating -> Active`).
  - JSON: Serialized `UserSessionActionReport` envelope (`{"code": 0, "data": UserSessionActionReport, "error": null}`).
- **Exit Codes**: `0` on success, `1` on FSM rejection, non-existent session, or persistence error, `2` on missing or unknown action/arguments.
- **Persistence**: Mutates in-memory store; atomically persists to disk when `--store` is specified.

#### 5. `create` (AIOS-specific)
- **Syntax**: `aiosh session create <spec_file_or_json> [--store <path>] [--json]`
- **Inputs**: Specification file path or inline JSON string satisfying `UserSessionSpec`.
- **Validation**: Enforces 1 MiB payload ceiling. Evaluates capacity bounds ($\le 32$ active sessions per user, $\le 1,024$ total sessions) and invariant rules.
- **Output**:
  - Text: Confirmation of session creation with initial `Initializing` state.
  - JSON: Result envelope containing `UserSessionActionReport` with action `create`.
- **Exit Codes**: `0` on creation success, `1` on capacity exhaustion, duplicate session ID, or persistence failure, `2` on invalid syntax, malformed JSON, or missing argument.
- **Persistence**: Inserts session into store; atomically persists to disk when `--store` is specified.

#### 6. Convenience Action Shortcuts (AIOS-specific)
- **Syntax**:
  - `aiosh session activate <session_id> [--store <path>] [--json]` (delegates to `action <id> activate`)
  - `aiosh session lock <session_id> [--store <path>] [--json]` (delegates to `action <id> lock`)
  - `aiosh session unlock <session_id> [--store <path>] [--json]` (delegates to `action <id> unlock`)
  - `aiosh session terminate <session_id> [--store <path>] [--json]` (delegates to `action <id> terminate`)
  - `aiosh session auth <session_id> [--store <path>] [--json]` (delegates to `action <id> authenticate`)
- **Inputs**: Target session identifier and optional store path.
- **Output & Exit Codes**: Identical to underlying `action` command.

---

## 2. Reuse vs. AIOS-Specific Extensions

### Reused Upstream Components
- `aiosh-core::session`: Core types (`UserSessionSpec`, `UserSessionStatus`, `UserSessionQuery`, `SessionState`, `SessionScope`, `SessionType`, `SessionClass`, `UserSessionAction`, `UserSessionStore`) and validation functions (`validate_session_id`, `validate_username`, `validate_user_session_spec`).
- `aiosh-core::session_service`: `UserSessionService` coordinator, `UserSessionActionReport`, seat arbitration, and atomic persistence.
- `classify_and_emit`: System-wide structured audit emission to SQLite WAL ring (`audit.db`) with SHA-256 hash chaining.

### AIOS-Specific Extensions
- Ergonomic convenience shortcuts (`status`, `activate`, `lock`, `unlock`, `terminate`, `auth`) providing alignment with `loginctl(1)` while integrating into the unified AIOS CLI framework.
- Dedicated `create` subcommand enabling non-interactive bootstrap registration.
- Structured JSON envelopes across all commands ensuring strict machine readability.

---

## 3. Audit Trail Guarantee (ADR-0035 §F-2)

Every command execution path unconditionally calls `classify_and_emit`:
```rust
classify_and_emit(
    &mut ctx,
    "session",
    subcommand_name,
    json_params,
    outcome,
    target_session_id,
    detail_message,
    "operator",
    None,
);
```
Guaranteeing complete non-repudiation audit row logging across both success and failure execution branches.

---

## 4. Acceptance Verification
- [x] Specification covers happy path, failure path, and audit effects for all subcommands.
- [x] Clear demarcations between reused upstream components and AIOS-specific extensions.
- [x] Document is fully reviewable without reading the implementation source code.
