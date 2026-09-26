# T-02267 Security Review: Grant Lifecycle Security Policy

**Task:** Security-review the security policy of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Security Policy  

---

## 1. Threat Modeling & Abuse Scenarios

| ID | Abuse Scenario | Attack Vector | Mitigation in Code | Status |
|---|---|---|---|---|
| **THREAT-1** | Arbitrary File Overwrite / Read | Attacker provides path traversal string (`../../etc/shadow`) in policy file path | `path_str.contains("..")` rejection in both `load_from_path` and `save_to_path` | ✅ Mitigated |
| **THREAT-2** | Memory Exhaustion (DoS) | Hostile actor attempts to load multi-gigabyte policy file | `metadata.len() > MAX_GRANT_POLICY_BYTES (64 KiB)` check before reading | ✅ Mitigated |
| **THREAT-3** | Privilege Escalation via Attenuation | Worker derives child grant with higher rights or prohibited rights (`Admin`) | Monotonicity check + `disallowed_delegation_rights` enforcement | ✅ Mitigated |
| **THREAT-4** | Zombie Credentials (Infinite Lifetime) | Actor attempts to issue non-expiring or century-long grant | `require_explicit_expiry` and `max_grant_duration_seconds` ceiling checks | ✅ Mitigated |
| **THREAT-5** | Delegation Depth Explosion | Deep recursive delegation tree causing stack exhaustion | `max_delegation_depth` bound (1..=8) and strict child < parent decrement | ✅ Mitigated |
| **THREAT-6** | Prohibited Subject Impersonation | Untrusted or anonymous actor claims capability grant | `prohibited_subject_patterns` glob filter (blocks `*anonymous*`, `*nobody*`, `*untrusted*`) | ✅ Mitigated |

---

## 2. Invariant Compliance
- **Fail-Closed Guarantee:** Under default `PepGrantEnforcementMode::Enforcing`, any policy failure rejects grant operations immediately.
- **Audit Compliance:** Non-compliant actions generate structured error codes (`GRANTPOL_ERR_*`) suitable for dispatch audit logging.
- **No Residual Bypasses:** Zero unhandled bypass paths identified in review.
