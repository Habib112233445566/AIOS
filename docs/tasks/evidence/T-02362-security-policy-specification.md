# Task Evidence: T-02362 (Audit Chain Extensions / security policy: Specification)

## 1. Metadata
- **Task ID:** `T-02362`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions Security Policy Specification
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 7: Security Policy (2/4) — Specification

---

## 2. Technical Specification

### 2.1 Enums, Structs, and Constants

```rust
pub const MAX_AUDIT_POLICY_VERSION_LEN: usize = 32;
pub const MAX_AUDIT_POLICY_DESC_LEN: usize = 512;
pub const MAX_PROHIBITED_ACTORS: usize = 128;
pub const MAX_PROHIBITED_TOOLS: usize = 128;
pub const MAX_SIGNATURE_REQUIRED_PREFIXES: usize = 64;
pub const MAX_ALLOWED_CAUSAL_LINKS_UPPER_BOUND: usize = 64;
pub const MAX_AUDIT_SECURITY_POLICY_BYTES: u64 = 64 * 1024; // 64 KiB
pub const MAX_PERMISSIBLE_TIMESTAMP_FUTURE_SKEW_SECS: u64 = 300; // 5 minutes

pub const AUDITPOL_ERR_VALIDATION: &str = "AUDITPOL_ERR_VALIDATION";
pub const AUDITPOL_ERR_DENIED: &str = "AUDITPOL_ERR_DENIED";
pub const AUDITPOL_ERR_SIGNATURE_REQUIRED: &str = "AUDITPOL_ERR_SIGNATURE_REQUIRED";
pub const AUDITPOL_ERR_TEMPORAL: &str = "AUDITPOL_ERR_TEMPORAL";
pub const AUDITPOL_ERR_IO: &str = "AUDITPOL_ERR_IO";
pub const AUDITPOL_ERR_PARSE: &str = "AUDITPOL_ERR_PARSE";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditPolicyMode {
    Enforcing,
    Permissive,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum AuditPolicyVerdict {
    Permit,
    PermitWithWarning { reason: String },
    Deny { reason: String, error_code: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainSecurityPolicy {
    pub version: String,
    pub mode: AuditPolicyMode,
    pub description: String,
    pub disallow_anonymous: bool,
    pub prohibited_actors: Vec<String>,
    pub prohibited_tools: Vec<String>,
    pub signature_required_prefixes: Vec<String>,
    pub max_allowed_causal_links: usize,
    pub allow_future_timestamps_max_secs: u64,
    pub valid_from_epoch_secs: Option<u64>,
    pub valid_until_epoch_secs: Option<u64>,
}
```

### 2.2 Method Invariants & Contract

1. **`AuditChainSecurityPolicy::default()`**:
   - `version`: `"1.0.0"`
   - `mode`: `AuditPolicyMode::Enforcing`
   - `description`: `"Default AIOS Audit Chain Security Policy"`
   - `disallow_anonymous`: `true`
   - `prohibited_actors`: `vec!["anonymous", "guest", "untrusted"]`
   - `prohibited_tools`: `vec!["raw_exec_bypass", "disable_pep", "drop_audit_chain"]`
   - `signature_required_prefixes`: `vec!["kernel:", "sec:", "admin:", "pep:"]`
   - `max_allowed_causal_links`: `32`
   - `allow_future_timestamps_max_secs`: `300`
   - `valid_from_epoch_secs`: `None`
   - `valid_until_epoch_secs`: `None`

2. **`validate(&self) -> Result<(), String>`**:
   - `version` length $\in [1, 32]$ and must start with `"1."`.
   - `description` length $\le 512$.
   - `prohibited_actors.len() <= 128`, `prohibited_tools.len() <= 128`.
   - `signature_required_prefixes.len() <= 64`.
   - `max_allowed_causal_links` must satisfy $1 \le \text{links} \le 64$.
   - If both `valid_from_epoch_secs` and `valid_until_epoch_secs` are present, `valid_from <= valid_until`.

3. **`evaluate_event(&self, event: &AuditRowInput, current_epoch_secs: u64) -> AuditPolicyVerdict`**:
   - If `self.mode == AuditPolicyMode::Disabled`, returns `AuditPolicyVerdict::Permit`.
   - Evaluates temporal validity (`valid_from_epoch_secs`, `valid_until_epoch_secs`).
   - Checks clock skew: `event.timestamp <= current_epoch_secs + self.allow_future_timestamps_max_secs`.
   - Checks anonymous provenance: if `disallow_anonymous` is true, rejects empty strings or `"anonymous"`.
   - Checks `prohibited_actors` and `prohibited_tools`.
   - Checks signature requirement: if `event.tool` starts with any prefix in `signature_required_prefixes`, requires valid Ed25519 signature in `signature` field (64-byte hex string / 128 hex chars) and non-empty `signer_pubkey` (32-byte hex string / 64 hex chars).
   - In `Enforcing` mode: returns `Deny`. In `Permissive` mode: returns `PermitWithWarning`.

4. **`evaluate_causal_links(&self, links: &[AuditCausalLinkInput]) -> AuditPolicyVerdict`**:
   - If `self.mode == AuditPolicyMode::Disabled`, returns `AuditPolicyVerdict::Permit`.
   - Verifies `links.len() <= self.max_allowed_causal_links`.
   - Verifies each `parent_event_hash` is 64 valid lowercase hex characters.

5. **`load_from_file<P: AsRef<Path>>(path: P)` and `save_to_file<P: AsRef<Path>>(&self, path: P)`**:
   - Standard file I/O with size check ($\le 64 \text{ KiB}$) and atomic persistence.

---

## 3. Acceptance Confirmation
- [x] Input, output, error cases, and persistence bounds defined.
- [x] Tri-mode evaluation (`Enforcing`, `Permissive`, `Disabled`) specified.
- [x] Invariant limits and error codes (`AUDITPOL_ERR_*`) documented.
