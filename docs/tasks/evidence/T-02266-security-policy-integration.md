# T-02266 Integration: Grant Lifecycle Security Policy

**Task:** Integrate the security policy of Grant Lifecycle with the surrounding system.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Security Policy  

---

## 1. What Was Integrated

1. **Service Layer Integration (`aiosh-core::pep_grant_service`):**
   - Configured `PepGrantService` with optional `PepGrantSecurityPolicy`.
   - Wired enforcement hooks directly into:
     - `issue_grant(&mut self, grant: PepGrant)`: Automatically checks `validate_grant(&grant)` before committing to storage.
     - `attenuate_grant(&mut self, ...)`: Automatically checks `validate_attenuation(&parent, &child)` to prevent forbidden delegation rights.

2. **Core Library Re-Exports (`aiosh-core::lib.rs`):**
   - Publicly exported `PepGrantSecurityPolicy`, `PepGrantEnforcementMode`, and policy error constants (`GRANTPOL_ERR_*`).

3. **Cross-Substrate Parity:**
   - Canonical JSON persistence format defined and tested with RFC 3339 timestamps and snake_case serde keys.
   - Tested under Rust unit and integration suites (`test_pep_grant_security_policy.rs`).

---

## 2. Acceptance Verification
- ✅ Feature reachable through production API `PepGrantService` and `aiosh_core`.
- ✅ End-to-end policy enforcement verified across issue and attenuate flows.
- ✅ Zero compiler warnings or errors in workspace.
