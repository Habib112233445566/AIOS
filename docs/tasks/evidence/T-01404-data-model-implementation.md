# T-01404: User Session Bootstrap - Data Model: Implementation

## Metadata
- **Task ID:** `T-01404`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Data Model Implementation (`code/aiosh-rust/aiosh-core::session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (4/10) — Data Model Implementation

---

## 1. Implementation Summary
Implemented complete validation logic and state store operations for the User Session Bootstrap data model in `code/aiosh-rust/aiosh-core/src/session.rs`, enforcing invariants `SB1..SB5`.

### Implemented Validation & Store Functions:
1. **`validate_session_id(id: &str) -> Result<(), String>`**:
   - Enforces `SB1`: Non-empty, $\le 64$ characters, begins with ASCII alphanumeric, only allowed characters `[a-zA-Z0-9_.-]`.
   - Rejects spaces, path slashes, null bytes, and path traversal sequences (`..`).

2. **`validate_username(name: &str) -> Result<(), String>`**:
   - Enforces `SB2`: Non-empty, $\le 32$ characters, begins with lowercase letter or underscore, allowed characters `[a-z0-9_-]`.
   - Rejects uppercase characters (POSIX convention), spaces, slashes, and metacharacters.

3. **`validate_user_session_spec(spec: &UserSessionSpec) -> Result<(), Vec<String>>`**:
   - Enforces `SB1` on `spec.session_id`.
   - Enforces `SB2` on `spec.username`.
   - Validates seat syntax (prefix `"seat"`, $\le 32$ chars).
   - Validates VTNR range `[1..12]` (mandatory for `Tty` session type).
   - Validates display syntax (prefix `":"`, mandatory for `X11` session type).
   - Validates remote host formatting if specified.
   - Enforces `SB4` on environment variables: $\le 256$ entries, keys `^[A-Z_][A-Z0-9_]{0,63}$`, values $\le 4096$ chars, no null bytes, and absolute path verification for `XDG_RUNTIME_DIR`.

4. **`transition_session_state(current: SessionState, action: UserSessionAction) -> Result<SessionState, String>`**:
   - Enforces `SB3`: Validates lifecycle state transitions against the formal transition matrix.
   - Supports `Initializing` $\to$ `Authenticating` $\to$ `Active` $\to$ `Locked` $\to$ `Terminating` $\to$ `Terminated`.
   - Rejects invalid or illegal transitions (e.g. attempting actions on a `Terminated` session).

5. **`validate_user_session_status(status: &UserSessionStatus) -> Result<(), Vec<String>>`**:
   - Validates identifier, username, non-empty timestamps, and consistency between `status.state` and `status.locked` / `status.scope`.

6. **`UserSessionStore` Lifecycle Operations**:
   - `add_session`: Validates spec and status, checks for duplicates, and enforces `SB5` capacity caps (max 32 sessions per user, max 1024 total sessions).
   - `get_session` / `get_spec`: Direct lookups by identifier.
   - `list_sessions`: Filtered discovery by username, state, session type, seat, and limit.
   - `apply_action`: Executes validated state machine transition and updates runtime status (e.g. locks, leader PID cleanup on termination).
   - `save_to_path` / `load_from_path`: Atomic disk persistence via temporary files with 10 MiB payload bounds.

---

## 2. Test Verification Output (`cargo test --lib session`)
```text
running 8 tests
test session::tests::test_state_machine_transitions_sb3 ... ok
test session::tests::test_session_id_syntax_sb1 ... ok
test session::tests::test_store_lifecycle_action_and_query ... ok
test session::tests::test_store_capacity_and_user_limits_sb5 ... ok
test session::tests::test_store_serialization_and_deserialization ... ok
test session::tests::test_environment_and_path_isolation_sb4 ... ok
test session::tests::test_username_syntax_sb2 ... ok
test session::tests::test_valid_spec_happy_path ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 336 filtered out; finished in 0.00s
```

### Regression Verification (`tools/test_service_suites.py`)
```text
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate, list, show/status, action, order)
[+] SS3 service MCP tool surface (validate, list, get, action, order)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)
[+] SS5 service configuration subsystem invariants, precedence & sizing (SC1..SC7)
[+] SS6 service automated integration tests (ST1..ST5)
[+] SS7 service security policy invariants & evaluation (SP1..SP6)
[+] SS8 service observability telemetry report & invariants (SO1..SO6)
[+] SS9 service documentation guide & invariants (D1..D6)
[+] SS10 service recovery subsystem & validation invariants (SR1..SR5)

PASS: service_suites criteria (SS1..SS10)
```

---

## 3. Acceptance Verification
- [x] Targeted unit tests pass with zero errors (8/8 in `session.rs`).
- [x] Zero regressions across existing test suite (all `SS1..SS10` criteria pass).
