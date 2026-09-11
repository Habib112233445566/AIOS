# T-01413: User Session Bootstrap - Core Service: Scaffold

## Metadata
- **Task ID:** `T-01413`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Core Service Scaffold (`code/aiosh-rust/aiosh-core::session_service`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (3/10) — Core Service Scaffold

---

## 1. Scaffold Deliverables

Created the module skeleton `code/aiosh-rust/aiosh-core/src/session_service.rs` and wired it into `aiosh-core` (`code/aiosh-rust/aiosh-core/src/lib.rs`).

### Defined Typed Interfaces & Data Structures:
1. **`UserSessionActionReport`**:
   - `session_id: String`
   - `action: UserSessionAction`
   - `previous_state: SessionState`
   - `new_state: SessionState`
   - `success: bool`
   - `error: Option<String>`
   - `timestamp: String`
2. **`UserSessionService`**:
   - Store container: `pub store: UserSessionStore`.
   - Function signatures with fail-loud scaffolding stubs (`unimplemented!()`):
     - `pub fn new() -> Self`
     - `pub fn empty() -> Self`
     - `pub fn create_session(&mut self, spec: UserSessionSpec) -> Result<UserSessionActionReport, String>`
     - `pub fn apply_action(&mut self, session_id: &str, action: UserSessionAction) -> Result<UserSessionActionReport, String>`
     - `pub fn query_sessions(&self, query: &UserSessionQuery) -> Vec<UserSessionStatus>`
     - `pub fn get_session(&self, session_id: &str) -> Option<&UserSessionStatus>`
     - `pub fn get_spec(&self, session_id: &str) -> Option<&UserSessionSpec>`
     - `pub fn list_sessions(&self) -> Vec<&UserSessionStatus>`
     - `pub fn update_idle(&mut self, session_id: &str, idle_seconds: u64) -> Result<(), String>`
     - `pub fn touch_activity(&mut self, session_id: &str) -> Result<(), String>`
     - `pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> io::Result<()>`
     - `pub fn load_from_path<P: AsRef<Path>>(path: P) -> io::Result<Self>`

---

## 2. Module Registration & Exports

- Registered in `code/aiosh-rust/aiosh-core/src/lib.rs`:
  ```rust
  pub mod session_service;
  pub use session_service::{UserSessionActionReport, UserSessionService};
  ```
- Included scaffolding unit tests in `session_service.rs`:
  - `test_session_action_report_scaffold_instantiation`: Verifies struct memory layout and Serde round-trip serialization/deserialization.
  - `test_user_session_service_scaffold_creation`: Verifies instantiation of empty service and store.
  - `test_create_session_scaffold_fails_loudly`: Confirms fail-loud behavior (`should_panic`).
  - `test_apply_action_scaffold_fails_loudly`: Confirms fail-loud behavior (`should_panic`).

---

## 3. Verification Outputs

### 1. Module Unit Tests (`cargo test --lib session_service`)
```text
running 4 tests
test session_service::tests::test_apply_action_scaffold_fails_loudly - should panic ... ok
test session_service::tests::test_create_session_scaffold_fails_loudly - should panic ... ok
test session_service::tests::test_user_session_service_scaffold_creation ... ok
test session_service::tests::test_session_action_report_scaffold_instantiation ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 344 filtered out; finished in 0.00s
```

### 2. Workspace Check (`cargo check --workspace`)
```text
    Checking aiosh-core v0.1.0
    Checking aiosh-mcp v0.1.0
    Checking aiosh-sandbox v0.1.0
    Checking aiosh-cli v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 23.86s
```

---

## 4. Acceptance Verification

- [x] Project builds with zero errors.
- [x] New interfaces exist and are referenced by at least one call site or test stub.
- [x] Fail-loud behavior verified on mutation endpoints.
