# Task Evidence: T-01902 - System Update Mechanism / data model: Specification

## 1. Overview
- **Task ID**: `T-01902`
- **Sub-Epic**: 1 (System Update Mechanism Data Model)
- **Goal**: Formally specify types, data models, slot representations, and validation interfaces for System Update Mechanism in `aiosh-core`.

---

## 2. Formal Specification

### 2.1 Enums & Basic Types
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateSlot {
    SlotA,
    SlotB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateChannel {
    Stable,
    Beta,
    Nightly,
    Development,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateState {
    Idle,
    Checking,
    Downloading,
    Verifying,
    Applying,
    ReadyToReboot,
    Verified,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartitionTarget {
    Rootfs,
    Kernel,
    Initramfs,
    FullBundle,
}
```

### 2.2 Structs & Validation Contracts
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateArtifact {
    pub target: PartitionTarget,
    pub file_name: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub update_id: String,
    pub version: String,
    pub channel: UpdateChannel,
    pub min_version: Option<String>,
    pub artifacts: Vec<UpdateArtifact>,
    pub signature: Option<String>,
    pub release_notes: String,
    pub published_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemSlotStatus {
    pub current_slot: UpdateSlot,
    pub target_slot: UpdateSlot,
    pub rollback_slot: Option<UpdateSlot>,
    pub slot_a_version: String,
    pub slot_b_version: String,
    pub slot_a_successful: bool,
    pub slot_b_successful: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemUpdateStatus {
    pub state: UpdateState,
    pub current_version: String,
    pub target_version: Option<String>,
    pub active_slot: UpdateSlot,
    pub progress_percent: u8,
    pub last_error: Option<String>,
    pub updated_at: String,
}
```

### 2.3 Invariant Enforcement Functions
- `validate_manifest(&self) -> Result<(), String>`:
  - `update_id` not empty and $\le 128$ chars.
  - `version` non-empty, $\le 64$ chars, valid semver-style format.
  - `artifacts` non-empty; each artifact has valid 64-char hex SHA-256 and `size_bytes > 0`.
- `validate_slot_status(&self) -> Result<(), String>`:
  - `current_slot != target_slot`.
  - Versions valid and non-empty.
- `can_transition(from: UpdateState, to: UpdateState) -> bool`:
  - Enforces linear state transitions and error/rollback branches.

Status: Specification completed. Ready for scaffolding in `T-01903`.
