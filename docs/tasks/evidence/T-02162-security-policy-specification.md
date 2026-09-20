# Task Evidence: T-02162 (PEP Decision Engine Security Policy: Specification)

## Overview
- **Task ID**: `T-02162`
- **Task Name**: security policy: Specification
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 7: Security Policy Subsystem
- **Timestamp**: 2026-09-21T01:20:00+05:00
- **Status**: COMPLETED

## Technical Specification: PEP Decision Security Policy Subsystem

### 1. Data Structures & Types
```rust
/// Enforcement mode governing PEP Decision Engine operations (PEPPOL1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepEnforcementMode {
    /// Enforcing: Decisions strictly enforce permits and denies. Fail-closed.
    Enforcing,
    /// Permissive (Audit-Only): Decisions are evaluated, but denied requests are permitted with an audit notice.
    Permissive,
    /// Disabled: Policy evaluation is bypassed entirely.
    Disabled,
}

/// Criticality level for PEP obligations (PEPPOL3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepObligationCriticality {
    /// Strict: If the obligation cannot be fulfilled, the decision must fail-closed (Deny).
    Strict,
    /// BestEffort: Obligation failures are logged but do not revoke a permit decision.
    BestEffort,
}

/// Security Policy governing the PEP Decision Engine (PEPPOL1..PEPPOL6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PepSecurityPolicy {
    pub version: String,
    pub mode: PepEnforcementMode,
    pub obligation_criticality: PepObligationCriticality,
    pub restricted_resource_prefixes: Vec<String>,
    pub valid_from_epoch_secs: Option<u64>,
    pub valid_until_epoch_secs: Option<u64>,
    pub description: String,
}
```

### 2. Error Code Taxonomy
- `PEPPOL_ERR_VALIDATION`: Malformed policy fields, invalid timestamps, or excessive length.
- `PEPPOL_ERR_PRIVILEGE`: Unprivileged caller attempting to permit restricted resources.
- `PEPPOL_ERR_TEMPORAL`: Policy evaluation outside configured temporal validity window.
- `PEPPOL_ERR_IO`: Filesystem errors, symlink detection, or atomic persistence failures.

### 3. Core Methods & Invariants

#### `validate(&self) -> Result<(), String>`
- Enforces:
  - `version` non-empty, $\le 32$ chars, alphanumeric/dots/hyphens.
  - `description` $\le 512$ chars.
  - `restricted_resource_prefixes` count $\le 64$, each prefix $\le 128$ chars.
  - If both `valid_from_epoch_secs` and `valid_until_epoch_secs` are set: `valid_from <= valid_until`.

#### `validate_rule_addition(&self, rule: &PepPolicyRule, caller_is_privileged: bool) -> Result<(), String>` (`PEPPOL2`)
- Prevents unprivileged agents from adding `Permit` rules that match restricted prefixes (`sys:*`, `sec:*`, `kernel:*`).

#### `enforce_decision(&self, decision: PepDecision, current_epoch_secs: u64) -> PepDecision` (`PEPPOL1`, `PEPPOL4`)
- Validates temporal validity. If expired or not yet active, returns default deny with `PEPPOL_ERR_TEMPORAL`.
- Applies enforcement mode:
  - `Enforcing`: Preserves `allowed == (effect == Permit)`.
  - `Permissive`: If `effect == Deny`, overrides `allowed = true` while preserving `effect: Deny` and appending an audit obligation noting permissive bypass.
  - `Disabled`: Sets `allowed = true`, `effect = Permit`, noting disabled mode.

#### `save_to_path(&self, path: &Path) -> Result<(), String>` & `load_from_path(path: &Path) -> Result<Self, String>` (`PEPPOL5`)
- Atomic persistence via `.tmp.<pid>` pattern.
- Rejection of symlinks and path traversal (`..`).
- Read bounded to `MAX_PEP_SECURITY_POLICY_BYTES = 64 * 1024` (64 KiB).
