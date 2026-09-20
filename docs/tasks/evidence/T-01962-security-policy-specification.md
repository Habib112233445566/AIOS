# Task Evidence: T-01962 (System Update / security policy: Specification)

## 1. Specification Overview
This document specifies the exact contract, data model, validation rules, evaluation logic, and persistence protocol for the AIOS System Update Security Policy Subsystem (`code/aiosh-rust/aiosh-core/src/system_update_policy.rs`).

## 2. Invariants Enforced (UPOL1 - UPOL6)

| Invariant | Name | Policy Field | Evaluation Condition & Rule ID | Fatal |
|---|---|---|---|---|
| **UPOL1** | Channel Authorization | `allowed_channels` | Manifest channel MUST be present in `allowed_channels`. Violation: `UPOL1_CHANNEL_DISALLOWED`. | Yes |
| **UPOL2** | Signature Enforcement | `require_signature`, `trusted_public_keys` | If `require_signature` is true, manifest MUST contain a signature. If `trusted_public_keys` non-empty, signature must match one trusted key. Violation: `UPOL2_SIGNATURE_MISSING`, `UPOL2_KEY_UNTRUSTED`. | Yes |
| **UPOL3** | Anti-Rollback / Downgrade Prevention | `disallow_downgrades` | If `disallow_downgrades` is true, candidate version MUST be $\ge$ current version based on semantic version comparison. Violation: `UPOL3_DOWNGRADE_ATTEMPT`. | Yes |
| **UPOL4** | Partition Target Governance | `allowed_partition_targets`, `required_partition_targets` | All manifest artifacts MUST belong to `allowed_partition_targets`. All `required_partition_targets` MUST be present. Violation: `UPOL4_TARGET_DISALLOWED`, `UPOL4_REQUIRED_TARGET_MISSING`. | Yes |
| **UPOL5** | Quota & Resource Caps | `max_payload_bytes`, `max_artifacts_count` | Total manifest payload bytes $\le \text{max\_payload\_bytes}$ and artifact count $\le \text{max\_artifacts\_count}$. Violation: `UPOL5_PAYLOAD_EXCEEDED`, `UPOL5_ARTIFACT_COUNT_EXCEEDED`. | Yes |
| **UPOL6** | Revocation Denylisting | `revoked_versions`, `revoked_update_ids` | Candidate version must not match `revoked_versions`. Candidate update_id must not match `revoked_update_ids`. Violation: `UPOL6_VERSION_REVOKED`, `UPOL6_UPDATE_ID_REVOKED`. | Yes |

## 3. Data Model & Types

### 3.1 `UpdatePolicyMode`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePolicyMode {
    Enforcing, // Any fatal violation results in verdict "deny"
    Audit,     // Fatal violations logged, verdict remains "audit"
    Permissive,// Violations recorded, verdict remains "allow"
}
```

### 3.2 `SystemUpdateSecurityPolicy`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemUpdateSecurityPolicy {
    pub mode: UpdatePolicyMode,
    pub allowed_channels: Vec<UpdateChannel>,
    pub require_signature: bool,
    pub trusted_public_keys: Vec<String>,
    pub disallow_downgrades: bool,
    pub allowed_partition_targets: Vec<PartitionTarget>,
    pub required_partition_targets: Vec<PartitionTarget>,
    pub max_payload_bytes: u64,
    pub max_artifacts_count: usize,
    pub revoked_versions: Vec<String>,
    pub revoked_update_ids: Vec<String>,
}
```

### 3.3 `UpdatePolicyViolation` & `UpdatePolicyReport`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdatePolicyViolation {
    pub rule_id: String,
    pub target: String,
    pub description: String,
    pub fatal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdatePolicyReport {
    pub verdict: String, // "allow", "deny", "audit"
    pub mode: UpdatePolicyMode,
    pub violations: Vec<UpdatePolicyViolation>,
    pub current_version: String,
    pub candidate_version: String,
    pub artifacts_evaluated: usize,
    pub total_payload_bytes: u64,
}
```

## 4. Operational Methods & Semantics
- `evaluate(&self, current_version: &str, manifest: &UpdateManifest) -> UpdatePolicyReport`:
  Pure evaluation function returning report. Verdict is computed based on mode:
  - If `mode == Enforcing` and any fatal violation exists -> `"deny"`.
  - If `mode == Audit` and any fatal violation exists -> `"audit"`.
  - Otherwise -> `"allow"`.
- `validate(&self) -> Result<(), String>`:
  Validates policy self-consistency (e.g. `allowed_channels` not empty, `max_payload_bytes` within 1MB..10GB, `max_artifacts_count` within 1..32).
- `from_file(path: &Path) -> Result<Self, String>` & `save_to_file(&self, path: &Path) -> Result<(), String>`:
  Atomic file persistence (`.tmp.<pid>`) with 1 MB size cap and `validate_policy_path()` hygiene check.

## 5. Error Codes
- `UPOL_VALIDATION_ERROR`: Policy configuration or semantic error.
- `UPOL_IO_ERROR`: Filesystem I/O failure.
- `UPOL_PARSE_ERROR`: JSON parsing/serialization failure.
- `UPOL_PATH_ERROR`: Path hygiene violation.
