# T-02263 Scaffold: Grant Lifecycle Security Policy

**Task:** Create the module skeleton and interfaces for the security policy of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Security Policy  

---

## 1. What Was Created

1. **Source File:** `code/aiosh-rust/aiosh-core/src/pep_grant_security_policy.rs`
   - Defines `PepGrantEnforcementMode` (`Enforcing`, `Permissive`, `Disabled`).
   - Defines `PepGrantSecurityPolicy` with:
     - `version: String`
     - `mode: PepGrantEnforcementMode`
     - `max_grant_duration_seconds: u64`
     - `max_delegation_depth: u32`
     - `disallowed_delegation_rights: Vec<CapabilityRight>`
     - `require_explicit_expiry: bool`
     - `prohibited_subject_patterns: Vec<String>`
     - `max_store_capacity: usize`
   - Defines methods: `validate()`, `validate_grant()`, `validate_attenuation()`, `load_from_path()`, `save_to_path()`.
   - Defines boundary constants and error codes (`GRANTPOL_ERR_*`).

2. **Module Integration:**
   - Registered `pub mod pep_grant_security_policy;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
   - Re-exported core structs and constants in `aiosh_core`.

---

## 2. Acceptance Verification
- ✅ Module skeleton and interfaces created.
- ✅ Re-exported and registered in `lib.rs`.
- ✅ Compiles cleanly under `aiosh-core`.
