# T-01512: Filesystem Layout - Core Service: Specification

## Metadata
- **Task ID:** `T-01512`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout Core Service (`code/aiosh-rust/aiosh-core::fs_layout_service`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (3/10) — Core Service Specification
- **Dependencies:** `T-01511` (Core Service Research)
- **Next Task:** `T-01513` (Filesystem Layout / core service: Scaffold)

---

## 1. Scope & Objective

This specification establishes the interface contract, invariants, error handling, and persistence semantics for the **Filesystem Layout Core Service** (`fs_layout_service.rs`) in `aiosh-core`. 

The service sits between the pure data model (`fs_layout.rs`) and external surfaces (CLI subcommands, MCP tools, and system boot/agent coordinators).

---

## 2. Reused vs. New Interfaces

### 2.1 Reused Interfaces (from `code/aiosh-rust/aiosh-core/src/fs_layout.rs`)
- `FilesystemLayoutSpec`: Canonical schema holding layout metadata, partitions, mounts, and directories.
- `MountPointSpec`: 6-field `fstab(5)` specification and validator.
- `PartitionSpec`: GPT partition table record with index, label, type GUID, capacity, and flags.
- `DirectorySpec`: Essential directory entry with mode, ownership, and UsrMerge symlink targets.
- `FsType`: Filesystem driver enum (`Ext4`, `Vfat`, `Tmpfs`, `Btrfs`, etc.).
- `PartitionType`: GPT partition type GUID abstraction (`EfiSystem`, `LinuxRoot`, `LinuxSwap`, etc.).
- `validate_filesystem_layout(&spec)`: FL1..FL5 invariant validation engine.

### 2.2 New Interfaces (introduced in `fs_layout_service.rs`)
- `FilesystemLayoutStore`: In-memory and serialized registry for managing layout specs and active pointer.
- `FilesystemLayoutService`: High-level operational coordinator handling probing, diffing, fstab reconciliation, and atomic persistence.
- `TargetEvaluation`: Detailed report evaluating target block device capacity, partition budgets, and warning/error diagnostics.
- `LayoutDiff`, `PartitionDiffItem`, `MountDiffItem`: Differential comparison model between two layouts identifying structural and destructive changes.

---

## 3. Detailed Data Structures & Signatures

```rust
use std::collections::BTreeMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::fs_layout::{
    DirectorySpec, FilesystemLayoutSpec, FsType, MountPointSpec, PartitionSpec, PartitionType,
};

/// In-memory and persistent registry for filesystem layout profiles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesystemLayoutStore {
    pub active_layout_id: String,
    pub layouts: BTreeMap<String, FilesystemLayoutSpec>,
}

impl FilesystemLayoutStore {
    /// Creates a store seeded with canonical UEFI and container presets.
    /// Active layout defaults to "aios-uefi-standard-v1".
    pub fn new() -> Self;

    /// Creates an empty store (active_layout_id is empty).
    pub fn empty() -> Self;

    /// Registers a new layout spec after running FL1..FL5 validation.
    /// Rejects duplicates unless explicitly intended.
    pub fn register_layout(&mut self, spec: FilesystemLayoutSpec) -> Result<(), String>;

    /// Retrieves an immutable reference to a layout by ID.
    pub fn get_layout(&self, id: &str) -> Option<&FilesystemLayoutSpec>;

    /// Lists all registered layouts sorted alphabetically by ID.
    pub fn list_layouts(&self) -> Vec<&FilesystemLayoutSpec>;

    /// Removes a layout by ID.
    /// Fails if attempting to remove the currently active layout or a non-existent layout.
    pub fn remove_layout(&mut self, id: &str) -> Result<FilesystemLayoutSpec, String>;

    /// Returns the active layout spec.
    pub fn get_active_layout(&self) -> Result<&FilesystemLayoutSpec, String>;

    /// Switches the active layout pointer to an existing registered layout ID.
    pub fn set_active_layout(&mut self, id: &str) -> Result<(), String>;
}

/// Evaluation report assessing target disk feasibility for a given layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetEvaluation {
    pub layout_id: String,
    pub target_disk_bytes: u64,
    pub required_disk_bytes: u64,
    pub partition_budget_bytes: u64,
    pub is_viable: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Differential summary between two filesystem layouts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutDiff {
    pub source_layout_id: String,
    pub target_layout_id: String,
    pub partitions_added: Vec<PartitionSpec>,
    pub partitions_removed: Vec<PartitionSpec>,
    pub partitions_modified: Vec<PartitionDiffItem>,
    pub mounts_added: Vec<MountPointSpec>,
    pub mounts_removed: Vec<MountPointSpec>,
    pub mounts_modified: Vec<MountDiffItem>,
    pub directories_added: Vec<DirectorySpec>,
    pub directories_removed: Vec<DirectorySpec>,
    pub destructive: bool,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionDiffItem {
    pub index: u32,
    pub old_label: String,
    pub new_label: String,
    pub old_size_mib: u64,
    pub new_size_mib: u64,
    pub old_type: PartitionType,
    pub new_type: PartitionType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MountDiffItem {
    pub path: String,
    pub old_device: String,
    pub new_device: String,
    pub old_fs: FsType,
    pub new_fs: FsType,
    pub old_options: Vec<String>,
    pub new_options: Vec<String>,
}

/// High-level filesystem layout service coordinator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesystemLayoutService {
    pub store: FilesystemLayoutStore,
}

impl FilesystemLayoutService {
    pub fn new() -> Self;
    pub fn empty() -> Self;
    pub fn store(&self) -> &FilesystemLayoutStore;
    pub fn store_mut(&mut self) -> &mut FilesystemLayoutStore;

    /// Evaluates whether a target block device meets requirements.
    pub fn probe_target(&self, layout_id: &str, target_disk_bytes: u64) -> Result<TargetEvaluation, String>;

    /// Computes fine-grained diff between two registered layouts.
    pub fn diff_layouts(&self, source_id: &str, target_id: &str) -> Result<LayoutDiff, String>;

    /// Synthesizes valid `/etc/fstab` text for the specified layout.
    pub fn export_fstab(&self, layout_id: &str) -> Result<String, String>;

    /// Imports external `/etc/fstab` content into a new layout profile.
    pub fn import_fstab_as_layout(
        &mut self,
        id: &str,
        name: &str,
        fstab_content: &str,
        base_layout_id: Option<&str>,
    ) -> Result<FilesystemLayoutSpec, String>;

    /// Atomically persists store state to a JSON file.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String>;

    /// Loads and validates store state from a JSON file.
    pub fn load_from_path(path: &Path) -> Result<Self, String>;
}
```

---

## 4. Invariant Enforcement & Business Rules

### 4.1 Invariant CS1: Store Coherence & Non-Empty State
- `store.active_layout_id` must point to an existing entry in `store.layouts`.
- Calling `remove_layout(id)` where `id == active_layout_id` is prohibited and returns `Err`.
- Built-in layouts (`aios-uefi-standard-v1` and `aios-container-minimal-v1`) are protected against accidental removal.

### 4.2 Invariant CS2: Target Geometry & Feasibility
- `probe_target(layout_id, target_disk_bytes)` checks:
  1. `target_disk_bytes >= spec.target_disk_min_bytes`.
  2. Total partition allocation: `sum(partition.size_mib) * 1024 * 1024 <= target_disk_bytes`.
  3. If partition allocation exceeds `target_disk_bytes`, `is_viable` is `false`, and an error is appended.
  4. If target disk has less than 10% slack headroom after partition allocation, a warning is emitted.

### 4.3 Invariant CS3: Destructive Mutation Flagging
- In `diff_layouts(source, target)`:
  - If any partition from `source` is missing in `target`, `destructive` is set to `true`.
  - If any partition's `size_mib` in `target` is smaller than in `source`, `destructive` is set to `true`.
  - If any mount's `fs_type` changes (e.g. `ext4` to `xfs`), `destructive` is set to `true`.
  - If the root mount `/` device or type changes, `destructive` is set to `true`.
  - If none of the above occur (e.g. only added non-root mounts, enlarged partitions, or added directories), `destructive` remains `false`.

### 4.4 Invariant CS4: Topological Fstab Ordering & Header Hygiene
- Generated fstab files must start with a `# /etc/fstab: static file system information` header.
- Root mount `/` must appear first, followed by child mounts ordered by path depth.
- Fields are column-aligned.

### 4.5 Invariant CS5: Atomic Persistence & Temp-File Replacement
- `save_to_path(path)` writes JSON to `<path>.tmp.<pid>` first, flushes and syncs to disk, and executes an atomic file rename.
- `load_from_path(path)` reads the file, parses JSON, validates the store and all contained layouts with `validate()`, and checks `active_layout_id` before returning.

---

## 5. Error Modes & Return Envelopes

All public service APIs return `Result<T, String>`. Error messages are explicit and actionable:
- Missing layout: `"layout with id '<id>' not found in store"`.
- Active removal: `"cannot remove active layout '<id>'; switch active layout first"`.
- Duplicate layout: `"layout with id '<id>' is already registered"`.
- Disk capacity deficit: `"target disk capacity (X bytes) is less than layout minimum (Y bytes)"`.
- Empty fstab import: `"fstab content contains no valid mount entries"`.
- Persistence failure: `"failed to write layout state to '<path>': <io_error>"`.

---

## 6. Audit Logging Integration

When actions are invoked through CLI or MCP surfaces:
- `fs_layout.register`: emitted upon `register_layout` (records `id`, `name`, `partitions_count`, `mounts_count`).
- `fs_layout.activate`: emitted upon `set_active_layout` (records `previous_active_id`, `new_active_id`).
- `fs_layout.remove`: emitted upon `remove_layout` (records `id`).
- `fs_layout.export_fstab`: emitted upon `export_fstab` (records `layout_id`, `output_lines`).
