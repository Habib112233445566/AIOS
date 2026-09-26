# T-02269 Documentation: Grant Lifecycle Security Policy

**Task:** Document the security policy of Grant Lifecycle for operators and agents.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Security Policy  

---

## 1. Overview

The Grant Lifecycle Security Policy (`PepGrantSecurityPolicy`) enforces constraints on capability grants, ensuring that delegations cannot exceed maximum allowed depth, prohibited rights (such as `Admin`) cannot be passed to delegates, grants have bounded lifetimes, and untrusted subjects are rejected.

---

## 2. Configuration Schema (JSON)

An operator or security administrator can define a grant security policy JSON file (e.g. `pep_grant_security_policy.json`):

```json
{
  "version": "1.0.0",
  "mode": "enforcing",
  "max_grant_duration_seconds": 2592000,
  "max_delegation_depth": 5,
  "disallowed_delegation_rights": ["admin"],
  "require_explicit_expiry": true,
  "prohibited_subject_patterns": ["*anonymous*", "*nobody*", "*untrusted*"],
  "max_store_capacity": 5000
}
```

### Parameter Reference
| Parameter | Type | Default | Description |
|---|---|---|---|
| `version` | `string` | `"1.0.0"` | Policy format version. |
| `mode` | `enum` | `"enforcing"` | Enforcement mode: `"enforcing"`, `"permissive"`, or `"disabled"`. |
| `max_grant_duration_seconds` | `number` | `2592000` (30 days) | Max allowable lifetime between `not_before` and `expires_at`. |
| `max_delegation_depth` | `number` | `5` | Upper limit on delegation depth (1..=8). |
| `disallowed_delegation_rights`| `string[]`| `["admin"]` | Capability rights forbidden from child grants. |
| `require_explicit_expiry` | `boolean` | `true` | When true, open-ended grants without expiry are rejected. |
| `prohibited_subject_patterns` | `string[]`| `["*anonymous*", ...]` | Subject patterns blocked from receiving grants. |
| `max_store_capacity` | `number` | `5000` | Hard cap on total grants in the registry. |

---

## 3. Rust API Usage

```rust
use aiosh_core::pep_grant_security_policy::{PepGrantSecurityPolicy, PepGrantEnforcementMode};
use aiosh_core::pep_grant_service::PepGrantService;

// Load policy from disk
let policy = PepGrantSecurityPolicy::load_from_path(".aios/pep_grant_policy.json")?;

// Attach policy to service
let mut service = PepGrantService::new().with_security_policy(policy);

// Issuing or attenuating grants now strictly adheres to policy
service.issue_grant(my_grant)?;
```

---

## 4. Known Limitations & Constraints
1. **Dynamic Pattern Matching:** Subject prohibited patterns currently use simple substring/glob wildcard matching; full regular expressions are deferred to Phase 3.
2. **Persistence Boundary:** The maximum policy file size is capped at 64 KiB to prevent denial-of-service.
3. **Immutability of Terminal States:** Transitioning from `revoked` or `expired` back to `active` is prohibited regardless of enforcement mode.
