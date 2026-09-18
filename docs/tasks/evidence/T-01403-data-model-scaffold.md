# T-01403: User Session Bootstrap - Data Model: Scaffold

## Metadata
- **Task ID:** `T-01403`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Data Model Scaffold (`code/aiosh-rust/aiosh-core::session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (3/10) — Data Model Scaffold

---

## 1. Scaffold Deliverables

Created module skeleton `code/aiosh-rust/aiosh-core/src/session.rs` and wired it into `aiosh-core` (`code/aiosh-rust/aiosh-core/src/lib.rs`).

### Defined Typed Interfaces & Data Structures:
- `SessionType`: Enum (`Tty`, `X11`, `Wayland`, `AiAgent`) with snake_case Serde bindings.
- `SessionClass`: Enum (`User`, `Greeter`, `LockScreen`, `Background`, `Agent`).
- `SessionState`: Enum (`Initializing`, `Authenticating`, `Active`, `Locked`, `Terminating`, `Terminated`).
- `SessionScope`: Enum (`Foreground`, `Background`).
- `UserSessionSpec`: Canonical specification for initializing or configuring user/agent sessions (session ID, username, UID, GID, session type, session class, seat, VTNR, display, remote host, environment variables).
- `UserSessionStatus`: Runtime status snapshot representing session state, leader PID, timestamps, idle tracking, and lock status.
- `UserSessionAction`: Enum (`Create`, `Authenticate`, `Activate`, `Lock`, `Unlock`, `Terminate`).
- `UserSessionQuery`: Search and filtering query struct (`username`, `state`, `session_type`, `seat`, `limit`).
- `UserSessionStore`: Store struct containing active sessions, specifications, and schema version.
- Typed validation function signatures with fail-loud scaffolding stubs:
  - `validate_session_id(id: &str) -> Result<(), String>`
  - `validate_username(name: &str) -> Result<(), String>`
  - `validate_user_session_spec(spec: &UserSessionSpec) -> Result<(), Vec<String>>`
  - `transition_session_state(current: SessionState, action: UserSessionAction) -> Result<SessionState, String>`
  - `validate_user_session_status(status: &UserSessionStatus) -> Result<(), String>`

---

## 2. Module Registration & Exports:
- Exported in `code/aiosh-rust/aiosh-core/src/lib.rs` under `pub mod session;` and re-exported all public session types and function signatures.
- Included unit tests in `session.rs`:
  - `test_session_types_scaffold_instantiation`: Validates memory layout and JSON serialization/deserialization.
  - `test_validate_session_id_scaffold_fails_loudly`: Confirms fail-loud scaffolding behavior.
  - `test_validate_username_scaffold_fails_loudly`: Confirms fail-loud scaffolding behavior.
  - `test_validate_user_session_spec_scaffold_fails_loudly`: Confirms fail-loud scaffolding behavior.
- Verified build and tests pass cleanly via `cargo check` and `cargo test --lib session`.

---

## 3. Verification Output (`cargo test --lib session`)
```text
running 4 tests
test session::tests::test_validate_user_session_spec_scaffold_fails_loudly - should panic ... ok
test session::tests::test_session_types_scaffold_instantiation ... ok
test session::tests::test_validate_username_scaffold_fails_loudly - should panic ... ok
test session::tests::test_validate_session_id_scaffold_fails_loudly - should panic ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 336 filtered out; finished in 0.00s
```

---

## 4. Acceptance Verification
- [x] Project builds with zero errors (`cargo check` passed).
- [x] New interfaces exist and are referenced by test stubs and re-exported in `lib.rs`.
- [x] Function stubs fail loudly (`unimplemented!`) as required.
