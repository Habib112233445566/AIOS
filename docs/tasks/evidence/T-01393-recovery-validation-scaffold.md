# T-01393: Init & Service Supervision Recovery & Validation Scaffold

**Date:** 2026-09-09  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Init & Service Supervision / Recovery & Validation  
**Task ID:** T-01393  

---

## 1. Executive Summary
Task `T-01393` created the module skeleton and interface definitions for the **Init & Service Supervision Recovery & Validation** subsystem in `code/aiosh-rust/aiosh-core/src/service_recovery.rs`. The module exposes types `ServiceRecoveryAction` and `ServiceValidationReport` with invariant validation methods (`validate_invariants`), along with typed signatures for `validate_service_store`, `recover_service_store_with_backup`, and `load_or_recover`. The module is exported via `aiosh-core/src/lib.rs` and builds with zero errors across the workspace.

---

## 2. Interface Definitions

### `code/aiosh-rust/aiosh-core/src/service_recovery.rs`
- **`ServiceRecoveryAction`:**
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub enum ServiceRecoveryAction {
      LoadedExisting,
      CreatedDefaultFresh,
      RecoveredFromBackup { backup_path: String, reason: String },
  }
  ```
- **`ServiceValidationReport`:**
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub struct ServiceValidationReport {
      pub store_path: String,
      pub total_services: usize,
      pub valid_services: usize,
      pub invalid_services: usize,
      pub errors: Vec<String>,
      pub warnings: Vec<String>,
      pub healthy: bool,
      pub evaluated_at: String,
  }
  ```
- **Invariant Checker:** `validate_invariants(&self) -> Result<(), String>` enforcing conservation laws `SR1` ($valid + invalid = total$), `SR2` ($healthy \iff errors.is\_empty() \land invalid == 0$), and `SR3` ($invalid > 0 \implies errors.len() \ge invalid$).
- **Function Stubs:**
  - `validate_service_store(store: &ServiceStore, store_path: &Path) -> ServiceValidationReport`
  - `recover_service_store_with_backup(path: &Path) -> (ServiceStore, Option<PathBuf>)`
  - `load_or_recover(path: &Path) -> Result<(ServiceStore, ServiceValidationReport, bool, Option<PathBuf>), String>`

---

## 3. Build & Test Verification

### Compilation: `cargo check --workspace`
```text
Checking aiosh-core v0.1.0
Checking aiosh-mcp v0.1.0
Checking aiosh-sandbox v0.1.0
Checking aiosh-cli v0.1.0
Finished `dev` profile [unoptimized + debuginfo] target(s)
Result: exit code 0
```

### Scaffold Test Execution: `cargo test -p aiosh-core service_recovery`
```text
running 1 test
test service_recovery::tests::test_scaffold_report_invariants ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; finished in 0.02s
```
