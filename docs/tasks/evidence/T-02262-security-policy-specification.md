# T-02262 Specification: Grant Lifecycle Security Policy

**Task:** Specify the exact contract for the security policy of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Security Policy  

---

## 1. Scope & Objectives

The Grant Lifecycle Security Policy (`PepGrantSecurityPolicy`) provides policy-driven governance over capability grant issuance, delegation depth, rights attenuation, and credential validity windows.

---

## 2. Data Types & Interface Specification

### 2.1 Enforcement Mode
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepGrantEnforcementMode {
    /// Enforcing: Invariant failures immediately reject grant issuance or delegation (Fail-closed).
    Enforcing,
    /// Permissive: Violations are audited/logged but operations are permitted.
    Permissive,
    /// Disabled: Policy enforcement is bypassed.
    Disabled,
}
```

### 2.2 Security Policy Structure
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepGrantSecurityPolicy {
    /// Policy schema version (e.g. "1.0.0").
    pub version: String,
    /// Operational mode governing enforcement.
    pub mode: PepGrantEnforcementMode,
    /// Maximum allowed grant lifetime in seconds (default: 2,592,000 = 30 days).
    pub max_grant_duration_seconds: u64,
    /// Absolute upper bound on delegation depth (1..=8, default: 5).
    pub max_delegation_depth: u32,
    /// Rights forbidden from being delegated (e.g., [CapabilityRight::Admin]).
    pub disallowed_delegation_rights: Vec<CapabilityRight>,
    /// Require that all grants declare an explicit `expires_at` timestamp.
    pub require_explicit_expiry: bool,
    /// Glob patterns for prohibited recipient subjects (e.g., ["*anonymous*", "*guest*"]).
    pub prohibited_subject_patterns: Vec<String>,
    /// Maximum number of grants allowed in the system registry (default: 5,000).
    pub max_store_capacity: usize,
}
```

### 2.3 Constants & Error Codes
| Constant / Code | Type | Value | Purpose |
|---|---|---|---|
| `MAX_GRANT_POLICY_BYTES` | `u64` | `65536` | Maximum file size for policy JSON |
| `DEFAULT_MAX_GRANT_DURATION_SECS` | `u64` | `2_592_000` | 30 days default lifetime |
| `MAX_PERMISSIBLE_DURATION_SECS` | `u64` | `31_536_000` | 365 days max allowed |
| `GRANTPOL_ERR_VALIDATION` | `&str` | `"GRANTPOL_ERR_VALIDATION"` | Malformed policy struct |
| `GRANTPOL_ERR_POLICY_VIOLATION` | `&str` | `"GRANTPOL_ERR_POLICY_VIOLATION"` | Grant rejected by policy |
| `GRANTPOL_ERR_DELEGATION_REJECTED` | `&str` | `"GRANTPOL_ERR_DELEGATION_REJECTED"` | Disallowed right in delegation |
| `GRANTPOL_ERR_LIFETIME_EXCEEDED` | `&str` | `"GRANTPOL_ERR_LIFETIME_EXCEEDED"` | Grant exceeds max allowed lifespan |
| `GRANTPOL_ERR_IO` | `&str` | `"GRANTPOL_ERR_IO"` | Disk or JSON serialize error |

---

## 3. Evaluation Contract

### 3.1 `validate_grant(&self, grant: &PepGrant) -> Result<(), String>`
1. **Mode Check:** If `mode == Disabled`, return `Ok(())`.
2. **Explicit Expiry:** If `require_explicit_expiry` is true and `grant.constraints.expires_at` is `None`, return `Err(GRANTPOL_ERR_POLICY_VIOLATION)`.
3. **Lifetime Bound:** If `expires_at` exceeds `max_grant_duration_seconds` relative to `not_before`, return `Err(GRANTPOL_ERR_LIFETIME_EXCEEDED)`.
4. **Subject Pattern:** If `grant.subject` matches any pattern in `prohibited_subject_patterns`, return `Err(GRANTPOL_ERR_POLICY_VIOLATION)`.
5. **Delegation Depth:** If `grant.constraints.max_delegation_depth > self.max_delegation_depth`, return `Err(GRANTPOL_ERR_POLICY_VIOLATION)`.
6. In `Permissive` mode, any failure logs a warning and returns `Ok(())`.

### 3.2 `validate_attenuation(&self, parent: &PepGrant, child: &PepGrant) -> Result<(), String>`
1. **Monotonicity:** Verify child rights are subset of parent rights.
2. **Forbidden Delegation Rights:** If child contains any right in `disallowed_delegation_rights`, return `Err(GRANTPOL_ERR_DELEGATION_REJECTED)`.
3. **Child Grant Validation:** Validate child grant against `validate_grant(child)`.

---

## 4. Persistence Contract
- Atomic write to temporary file before rename.
- Rejection of path traversal (`..`).
- Strict 64 KiB file size limit on load.

---

## 5. Acceptance Verification
- ✅ Inputs, outputs, error cases, and boundaries specified.
- ✅ Reuses `PepGrant`, `CapabilityRight` without modifying their invariants.
- ✅ Covers happy path, failure paths, and audit effects.
