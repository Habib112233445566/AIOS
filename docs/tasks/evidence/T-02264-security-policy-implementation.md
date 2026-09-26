# T-02264 Implementation: Grant Lifecycle Security Policy

**Task:** Implement the minimal working behavior for the security policy of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Security Policy  

---

## 1. What Was Implemented

1. **`PepGrantSecurityPolicy` Full Implementation:**
   - Operational modes: `Enforcing` (fail-closed), `Permissive` (audit-only), `Disabled` (bypass).
   - Invariant validation: `max_grant_duration_seconds` bounded between 60s and 365d, `max_delegation_depth` bounded 1..=8, `max_store_capacity` bounded 1..=50,000.
   - Grant evaluation (`validate_grant`):
     - Prohibited subject patterns matching.
     - Mandatory explicit expiration timestamp enforcement.
     - Delegation depth ceiling enforcement.
     - Maximum lifetime duration enforcement.
   - Attenuation evaluation (`validate_attenuation`):
     - Monotonic rights inheritance.
     - Disallowed delegation rights prevention (e.g. `CapabilityRight::Admin`).
     - Strict depth decrement validation (child < parent).
   - Atomic persistence: `load_from_path` with 64 KiB ceiling and path traversal guard, `save_to_path` via tempfile atomic rename.

2. **Integration into `PepGrantService`:**
   - Added `policy: Option<PepGrantSecurityPolicy>` to `PepGrantService`.
   - Added builder `with_security_policy` and accessors `set_security_policy` / `security_policy`.
   - Wired policy enforcement directly into `issue_grant()` and `attenuate_grant()`.

---

## 2. Acceptance Verification
- ✅ Working implementation matches specification contract.
- ✅ Invariants fail-closed under `Enforcing` mode.
- ✅ Full workspace passes `cargo check -p aiosh-core` in 29.83s with zero errors.
