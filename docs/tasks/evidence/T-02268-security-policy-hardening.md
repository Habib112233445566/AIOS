# T-02268 Hardening: Grant Lifecycle Security Policy

**Task:** Harden the security policy of Grant Lifecycle against failure and misuse.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Security Policy  

---

## 1. Hardening Measures Implemented

1. **Strict File Size Bounds:**
   - Enforces `MAX_GRANT_POLICY_BYTES` (64 KiB) limit prior to buffer allocation.
   - Prevents denial-of-service via massive policy documents.

2. **Atomic Persistence with Crash Consistency:**
   - `save_to_path()` generates an isolated temporary file incorporating the current process PID (`.tmp.<pid>`).
   - Flushes buffers to disk prior to renaming.
   - Performs atomic file rename (`fs::rename`) guaranteeing that an incomplete write never corrupts the existing policy file.

3. **Defensive Path Validation:**
   - Canonical path checking rejects any input containing relative traversal sequences (`..`).

4. **Structured Error Hierarchy:**
   - Every failure emits an explicit error string tagged with a domain constant:
     - `GRANTPOL_ERR_VALIDATION`: Schema parameter out of bounds.
     - `GRANTPOL_ERR_POLICY_VIOLATION`: Subject or depth rule violation.
     - `GRANTPOL_ERR_DELEGATION_REJECTED`: Forbidden right in delegation.
     - `GRANTPOL_ERR_LIFETIME_EXCEEDED`: Duration exceeds window.
     - `GRANTPOL_ERR_IO`: Disk I/O or JSON parse failure.

5. **Resource Cleanup:**
   - File handles explicitly closed (`drop(f)`) before rename.
   - No dangling temporary files left in standard failure modes.

---

## 2. Acceptance Verification
- ✅ Failure modes produce explicit, auditable errors.
- ✅ Zero temporary file or memory leaks on error paths.
