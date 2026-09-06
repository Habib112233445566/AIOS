# T-01363: Init & Service Supervision - Security Policy: Scaffold

## Metadata
- **Task ID:** `T-01363`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Security Policy
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Scope & Objective
Create the initial module skeleton and public interfaces for the Init & Service Supervision Security Policy subsystem (`code/aiosh-rust/aiosh-core/src/service_policy.rs`) and export them in `lib.rs`.

---

## 2. Changes Made
1. **Module Creation**:
   - Created `code/aiosh-rust/aiosh-core/src/service_policy.rs` defining:
     - `ServicePolicyMode` enum (`Enforcing`, `Audit`, `Permissive`).
     - `ServiceSecurityPolicy` struct with configurable rules (`prohibited_services`, `prohibited_exec_paths`, `disallow_root`, `allowed_root_services`, `require_service_user`, `disallow_env_vars`, `allowed_service_types`, `max_env_vars`, `max_timeout_secs`).
     - `ServicePolicyViolation` and `ServicePolicyVerdict` structs.
     - Typed interface signatures:
       - `validate(&self) -> Result<(), String>`
       - `evaluate_spec(&self, spec: &ServiceSpec) -> ServicePolicyVerdict`
       - `evaluate_store(&self, store: &ServiceStore) -> Vec<ServicePolicyVerdict>`
       - `from_file<P: AsRef<Path>>(path: P) -> Result<Self, String>`
       - `from_source<F: Fn(&str) -> Option<String>>(lookup: F) -> Result<Self, String>`
       - `from_env() -> Result<Self, String>`
       - `resolve(custom_path: Option<&str>) -> Result<Self, String>`
2. **Re-exports in `lib.rs`**:
   - Added `pub mod service_policy;`
   - Re-exported `ServicePolicyMode`, `ServicePolicyVerdict`, `ServicePolicyViolation`, `ServiceSecurityPolicy`.
3. **Compilation Verification**:
   - Ran `cargo check --manifest-path code/aiosh-rust/Cargo.toml` across `aiosh-core`, `aiosh-sandbox`, `aiosh-mcp`, and `aiosh-cli`.
   - Build exited cleanly with exit code 0.
