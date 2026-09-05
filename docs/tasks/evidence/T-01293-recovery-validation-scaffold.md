# T-01293: Package Management Recovery & Validation Scaffold

**Date:** 2026-09-05  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Package Management / Recovery & Validation  
**Task ID:** T-01293  

---

## 1. Scaffold Overview
Task `T-01293` creates the module skeleton and typed interfaces for the **Recovery & Validation** subsystem of Package Management.

---

## 2. Module Definition (`package_recovery.rs`)
- File path: `code/aiosh-rust/aiosh-core/src/package_recovery.rs`.
- Declared in `code/aiosh-rust/aiosh-core/src/lib.rs` as `pub mod package_recovery;`.
- Re-exported types and methods:
  - `PackageValidationReport`
  - `validate_package_store`
  - `recover_package_store_with_backup`
  - `load_or_recover`

---

## 3. Data Structure & Invariant Checks
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageValidationReport {
    pub store_path: String,
    pub total_packages: usize,
    pub valid_packages: usize,
    pub invalid_packages: usize,
    pub errors: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}
```
Includes `validate_invariants(&self) -> Result<(), String>` enforcing:
- **`RV1`**: `valid_packages + invalid_packages == total_packages`
- **`RV2`**: `healthy == (errors.is_empty() && invalid_packages == 0)`
- **`RV3`**: `errors.len() >= invalid_packages`

---

## 4. Compilation Verification
The module and its exports were integrated into `aiosh-core` and verified with `cargo check -p aiosh-core`.
