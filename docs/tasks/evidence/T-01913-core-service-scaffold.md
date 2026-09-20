# Task Evidence: T-01913 - System Update Mechanism / core service: Scaffold

## 1. Overview
- **Task ID**: `T-01913`
- **Sub-Epic**: 2 (System Update Mechanism Core Service)
- **Goal**: Scaffold `SystemUpdateService` and `SystemUpdateServiceConfig` structures, method signatures, and export bindings in `aiosh-core`.

---

## 2. Scaffolded Structure
1. **Module Source**: `code/aiosh-rust/aiosh-core/src/system_update_service.rs`
   - `SystemUpdateServiceConfig`: `state_dir`, `staging_dir`, `max_payload_bytes`, `auto_rollback_on_failure`.
   - `SystemUpdateService`: `config`, `slot_status`, `update_status`, `active_manifest`, `staged_artifacts`.
   - Method signatures:
     - `new(current_version, active_slot, config, timestamp)`
     - `check_manifest(&mut self, manifest: UpdateManifest)`
     - `stage_artifact(&mut self, target: PartitionTarget, data: &[u8])`
     - `verify_staged(&mut self)`
     - `apply_update(&mut self)`
     - `confirm_boot(&mut self, running_version: &str)`
     - `rollback(&mut self)`
     - `fail(&mut self, reason: impl Into<String>)`
     - `save_state_to_dir(&self, dir: &Path)`
2. **Module Registration**: `code/aiosh-rust/aiosh-core/src/lib.rs`
   - `pub mod system_update_service;`
   - `pub use system_update_service::{SystemUpdateService, SystemUpdateServiceConfig};`

---

## 3. Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core` passed with zero errors and zero warnings in 5.53s.
