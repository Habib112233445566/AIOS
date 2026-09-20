# T-01693: Kernel Module Management Recovery & Validation Scaffold

## Sub-Epic
Kernel Module Management / Recovery & Validation (T-01693)

## Objective
Create the module skeleton and interfaces for the recovery & validation of Kernel Module Management under `code/aiosh-rust/aiosh-core/src/kernel_module_recovery.rs` and register it in `code/aiosh-rust/aiosh-core/src/lib.rs`.

## Scaffold Summary
1. **Source File Created**:
   - `code/aiosh-rust/aiosh-core/src/kernel_module_recovery.rs`
2. **Types & Signatures Defined**:
   - `KernelModuleValidationReport`: Structured integrity report with fields `store_path`, `total_rules`, `valid_rules`, `invalid_rules`, `total_autoload`, `valid_autoload`, `invalid_autoload`, `errors`, `healthy`, `recovered`, `backup_path`, `evaluated_at`.
   - `validate_invariants(&self) -> Result<(), String>`: Verifies internal invariants KR1, KR2, KR3.
   - `validate_kernel_module_store(store: &KernelModuleStore, store_path: &Path) -> KernelModuleValidationReport`: In-memory deep structural and conflict validation.
   - `check_store_file(path: &Path) -> Result<KernelModuleValidationReport, String>`: Read-only evaluation of store file integrity on disk.
   - `recover_store_file(path: &Path) -> Result<KernelModuleValidationReport, String>`: Automated non-destructive recovery with timestamped backup.
   - `create_timestamped_backup(path: &Path) -> Result<String, String>`: Timestamped quarantine backup mechanism.
3. **Crate Registration**:
   - Registered `pub mod kernel_module_recovery;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
4. **Build Verification**:
   - Verified via `cargo check -p aiosh-core` with zero errors and zero warnings.
