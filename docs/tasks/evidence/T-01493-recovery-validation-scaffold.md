# T-01493: User Session Bootstrap Recovery & Validation Scaffold

**Date:** 2026-09-16  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** User Session Bootstrap / Recovery & Validation  
**Task ID:** T-01493  

---

## 1. Scaffold Implementation Overview

Task `T-01493` implements the module skeleton and interfaces for the **Recovery & Validation** subsystem of User Session Bootstrap in `code/aiosh-rust/aiosh-core/src/session_recovery.rs` and exports it in `code/aiosh-rust/aiosh-core/src/lib.rs`.

---

## 2. Implemented Types & Signatures

### 2.1 Types
- **`SessionRecoveryAction`**:
  ```rust
  pub enum SessionRecoveryAction {
      LoadedExisting,
      CreatedDefaultFresh,
      RecoveredFromBackup { backup_path: String, reason: String },
  }
  ```
- **`SessionValidationReport`**:
  ```rust
  pub struct SessionValidationReport {
      pub store_path: String,
      pub total_sessions: usize,
      pub valid_sessions: usize,
      pub invalid_sessions: usize,
      pub errors: Vec<String>,
      pub warnings: Vec<String>,
      pub healthy: bool,
      pub evaluated_at: String,
  }
  ```

### 2.2 Mathematical Invariants (`SSR1..SSR3`)
`SessionValidationReport::validate_invariants(&self)` enforces:
- `SSR1`: `self.valid_sessions + self.invalid_sessions == self.total_sessions`
- `SSR2`: `self.healthy == (self.errors.is_empty() && self.invalid_sessions == 0)`
- `SSR3`: `self.invalid_sessions > 0 => self.errors.len() >= self.invalid_sessions`

### 2.3 Functions
- `pub fn validate_session_store(store: &UserSessionStore, store_path: &Path) -> SessionValidationReport`
- `pub fn recover_session_store_with_backup(path: &Path) -> (UserSessionStore, PathBuf)`
- `pub fn load_or_recover(path: &Path) -> Result<(UserSessionService, SessionValidationReport, bool, Option<PathBuf>), String>`

---

## 3. Verification & Build Confirmation

- `cargo test -p aiosh-core session_recovery`:
  ```
  running 1 test
  test session_recovery::tests::test_session_recovery_scaffold_interfaces ... ok
  test result: ok. 1 passed; 0 failed
  ```
- Module exported in `lib.rs`: `pub mod session_recovery;`.
- Workspace compiles with zero errors.
