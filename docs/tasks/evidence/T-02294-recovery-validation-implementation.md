# T-02294: Grant Lifecycle Recovery & Validation Implementation

## Overview
This task documents the implementation of the PEP Grant Store Recovery and Invariant Validation engine in `code/aiosh-rust/aiosh-core/src/pep_grant_recovery.rs`.

## Key Capabilities Implemented

### 1. In-Memory Grant Graph Validation (`validate_grants`)
- **Cycle Detection**: Walks ancestor links via `parent_grant_id` with a visited hash set to identify circular delegation cycles ($A \to B \to A$) in $O(V)$ time.
- **Orphan Detection**: Validates that all non-null parent references resolve to an existing grant in the active registry. Emits `PepGrantIssueCode::OrphanGrant`.
- **Attenuation Enforcement**:
  - Verifies parent grant possesses `CapabilityRight::Delegate`.
  - Verifies child rights are a strict subset of parent rights.
  - Verifies child max delegation depth is strictly less than parent delegation depth.
- **Cascade Revocation Synchronization**:
  - Checks if a parent is marked `Revoked` while a descendant is marked `Active` or `Suspended`. Flags `CascadeDesync`.
- **Temporal Consistency**:
  - Validates `not_before <= expires_at`. Emits `TimeInversion` if inverted.
  - Detects active grants whose `expires_at` has already passed, emitting `ExpiredActive` warnings.

### 2. File Store Validation (`validate_store_file`)
- Validates file path hygiene via `validate_grant_service_path` (rejecting path traversal `..`, control characters, non-json extensions).
- Enforces storage capacity ceiling (`MAX_GRANT_SERVICE_STORE_SIZE = 10 MiB`).
- Supports both service object format (`{"grants": { ... }}`) and array format (`[...]`).
- Returns structured `PepGrantValidationReport` detailing total grants, healthy count, issue list, validity status, and auto-repair feasibility.

### 3. Non-Destructive Store Recovery (`recover_store_file`)
- **Atomic Backup**: Writes point-in-time snapshot to `<path>.bak.<timestamp>`.
- **Corrupt File Quarantine**: If JSON syntax is fatally corrupt, quarantines the bad file to `<path>.quarantine.<timestamp>.json` and initializes a pristine empty grant store.
- **Auto-Repair Operations**:
  - Automatically revokes orphan grants (`action: "revoked_orphan"`).
  - Transitively cascades revocation to all descendants of revoked parents (`action: "reconciled_cascade"`).
  - Sweeps and marks expired active grants as `Expired` (`action: "auto_expired"`).
- **Atomic Replace**: Writes modified store to temporary file `<path>.tmp.<pid>` and performs atomic replacement via `fs::rename`.
- **Post-Validation**: Re-runs the validation engine on repaired store to verify healthy state.
