# Task Evidence: T-02092 (recovery & validation: Specification)

## Overview
- **Task ID**: T-02092
- **Sub-Epic**: Sub-Epic 10: Capability Model Recovery & Validation Subsystem
- **Component**: `aiosh-core::capability_recovery`
- **Objective**: Formally specify invariants (`CAPREC1..CAPREC6`), data structures, recovery actions, quarantine mechanisms, and validation algorithms.

## Formal Invariants (`CAPREC1..CAPREC6`)

| Invariant | Name | Specification |
|---|---|---|
| **`CAPREC1`** | **Partition Invariant** | `valid_capabilities + invalid_capabilities == total_capabilities`. Every capability in the store must be classified as either valid or invalid. |
| **`CAPREC2`** | **Health Consistency** | `healthy == (errors.is_empty() && invalid_capabilities == 0)`. The store is healthy if and only if no errors were encountered and zero invalid capabilities exist. |
| **`CAPREC3`** | **Lineage Integrity** | Every capability with `parent_id` must resolve to an existing parent capability. Cycles or dangling parent references are classified as critical integrity errors. |
| **`CAPREC4`** | **Monotonic Attenuation Check** | Every child capability must satisfy: `child.rights.is_subset(parent.rights)` and `parent.matches_scope(&child.scope)`. |
| **`CAPREC5`** | **Non-Destructive Quarantine** | Damaged, unreadable, or invalid store files must be preserved in a timestamped backup (`<file>.bak.<timestamp>[.<counter>]`) with mode 0600 before creating a clean default store. |
| **`CAPREC6`** | **Serialization & Traceability** | Reports and actions must serialize to deterministic JSON with structured error messages detailing the exact capability ID and invariant violated. |

## Data Structures Specification

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityRecoveryAction {
    LoadedExisting,
    CreatedDefaultFresh,
    RecoveredFromBackup { backup_path: String, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityValidationReport {
    pub store_path: String,
    pub total_capabilities: usize,
    pub valid_capabilities: usize,
    pub invalid_capabilities: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub healthy: bool,
    pub evaluated_at: String,
}

impl CapabilityValidationReport {
    pub fn validate_invariants(&self) -> Result<(), String>;
}

pub fn create_backup_file(path: &Path) -> PathBuf;
pub fn validate_capability_store(service: &CapabilityService, store_path: &Path) -> CapabilityValidationReport;
pub fn recover_capability_store(store_path: &Path) -> Result<(CapabilityService, CapabilityRecoveryAction, CapabilityValidationReport), String>;
```
