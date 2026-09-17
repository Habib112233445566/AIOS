# T-01511: Filesystem Layout - Core Service: Research

## Metadata
- **Task ID:** `T-01511`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout Core Service (`code/aiosh-rust/aiosh-core::fs_layout_service`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (2/10) — Core Service Research
- **Dependencies:** `T-01510` (Data Model Verification & Evidence)
- **Next Task:** `T-01512` (Filesystem Layout / core service: Specification)

---

## 1. Executive Summary & Objectives

Following the completion of the Filesystem Layout data model (`T-01501..T-01510`), Task `T-01511` establishes the architectural foundation, upstream constraints, authoritative references, and engineering decisions for the **Filesystem Layout Core Service** (`fs_layout_service.rs`).

The core service coordinates layout lifecycle operations:
1. **Registry & Store Management**: Managing a catalog of target filesystem layout profiles (`FilesystemLayoutSpec`), seeded with reference configurations (`aios-uefi-standard-v1`, `aios-container-minimal-v1`), tracking the active layout, and enabling dynamic layout registration and retirement.
2. **Target Disk Capacity & Feasibility Evaluation**: Validating whether target storage geometry (physical/virtual block devices) satisfies minimum footprint budgets, partition sector allocations, and alignment requirements.
3. **Layout Differential Analysis (`Diff`)**: Computing fine-grained deltas between active and candidate layouts across partitions, mounts, and directory trees, flagging high-risk/breaking modifications (e.g. root filesystem reformatting, partition shrinkage).
4. **Fstab Synthesis & Reconciliation**: Orchestrating `/etc/fstab` generation, safe non-destructive dry-run exports, and parsing/reconciling external fstab configurations against internal models.
5. **Atomic State Persistence**: Storing layout configuration state to disk atomically using temporary file renaming patterns (`write-temp-then-rename`) to protect against filesystem corruption during abrupt system power failures.

---

## 2. Authoritative Sources & Upstream Standards

1. **Linux Programmer's Manual - `fstab(5)` & `mount(8)`**:
   - *Citation*: man(5) fstab; man(8) mount; `util-linux` libmount documentation.
   - *Requirements*: `/etc/fstab` contains fixed-format description of filesystems to mount. The pass number (`fs_passno`) dictates `fsck` sequence during boot (root `/` = 1, others = 2, virtual/pseudo = 0).
   - *Mount Options*: Standard options include `defaults`, `rw`, `ro`, `nosuid`, `nodev`, `noexec`, `relatime`. The order of entries determines mounting sequence; parent directories must be mounted before their child mount points.

2. **freedesktop.org & systemd Mount Units**:
   - *Citation*: `systemd.mount(5)` and `systemd-fstab-generator(8)`.
   - *Architecture*: systemd converts `/etc/fstab` entries dynamically into ephemeral `.mount` system units at boot. The dependency graph (`Requires=`, `After=`) is inferred strictly from the hierarchical mount path prefixes (e.g., `/boot/efi` requires and runs after `/boot`, which requires and runs after `/`).
   - *Service Invariant*: The core service must ensure that mount lists maintain topological hierarchy order so that generated fstab entries translate into acyclic, valid systemd unit dependencies.

3. **UEFI Specification v2.10 & GPT Partitioning Standards**:
   - *Citation*: UEFI Forum v2.10 §13.3; DISA STIG Linux Partitioning Guidance.
   - *Storage Rules*: GUID Partition Tables (GPT) use 128-bit GUIDs to identify partition functions. ESP must be FAT32 with GUID `c12a7328-f81f-11d2-ba4b-00a0c93ec93b` and 1 MiB boundary sector alignment.
   - *Service Invariant*: Target evaluation logic must verify that partition boundaries fit within the target block device size and that partitions do not overlap.

4. **POSIX.1-2017 Atomic File Replacement (`rename(2)`)**:
   - *Citation*: IEEE Std 1003.1-2017 (POSIX.1) §`rename`.
   - *Rule*: Writing state directly to a target file risks leaving a truncated, zero-byte, or corrupt file if the host crashes or loses power mid-write.
   - *Pattern*: Atomic persistence writes the serialized payload to a sibling temporary file (e.g., `<file>.tmp.<pid>`) and executes an atomic `rename(tmp, target)`.

5. **Existing AIOS Core Services Prior Art**:
   - *Reference*: `code/aiosh-rust/aiosh-core/src/session_service.rs`, `base_image_service.rs`, `package_service.rs`.
   - *Conventions*:
     - Stores are modeled with in-memory `BTreeMap` or `HashMap` structures.
     - Constructors offer `new()` (seeded with canonical defaults) and `empty()` (for sterile test sandboxes).
     - Services expose explicit mutation methods returning `Result<(), String>`.
     - Invariants and policy checks are executed prior to state modification.
     - Consequential actions generate structured reports and emit audit records.

---

## 3. Facts vs. Assumptions

| Domain | Verified Fact | Technical Assumption |
|---|---|---|
| **Storage Mutation** | The core service is a control and metadata orchestrator; it does not directly format block devices (`mkfs`) or invoke `fdisk` without explicit, gated operator command. | The core service can fully validate layouts, calculate diffs, check disk constraints, and synthesize fstab files in pure user space without root privileges. |
| **Active Layout** | A running host or build target operates under exactly one active filesystem layout at any given point in time. | The service store should explicitly track `active_layout_id: String`, defaulting to `aios-uefi-standard-v1`, and reject activating non-existent layout IDs. |
| **Layout Diffing** | Administrators and autonomous agents need to assess the risk of changing layouts prior to executing partition restructuring. | A typed `LayoutDiff` structure can categorize changes into Partition, Mount, and Directory deltas, computing a `destructive` severity flag for unsafe mutations. |
| **Target Probing** | Target installation media varies in capacity (e.g., 16 GiB USB, 64 GiB SSD, 500 GiB NVMe). | The service can evaluate a candidate layout against a target disk capacity in bytes (`probe_target(disk_bytes)`) and report detailed budget shortfalls. |
| **fstab Management** | Systems require consistent `/etc/fstab` files formatted cleanly with comments, metadata, and column alignment. | The service can serialize layouts to fstab text and reconcile live fstab entries with layout specifications. |
| **Persistence** | System daemon state in `/etc/aios` must survive crashes without data loss. | Serializing `FilesystemLayoutStore` to JSON and writing atomically via temporary file replacement guarantees crash-safe state persistence. |

---

## 4. Architectural Design for `FilesystemLayoutService`

### 4.1 Core Types & Entities

```
+-------------------------------------------------------------+
|                FilesystemLayoutService                     |
+-------------------------------------------------------------+
| - store: FilesystemLayoutStore                             |
| - active_layout_id: String                                 |
+-------------------------------------------------------------+
         |
         +--> FilesystemLayoutStore
         |    - layouts: BTreeMap<String, FilesystemLayoutSpec>
         |
         +--> Evaluation & Diff Engine
         |    - probe_target(layout_id, disk_bytes) -> TargetEvaluation
         |    - diff_layouts(source_id, target_id) -> LayoutDiff
         |
         +--> Fstab Orchestrator
         |    - generate_fstab(layout_id) -> Result<String, String>
         |    - reconcile_fstab(layout_id, fstab_content) -> ReconcileReport
         |
         +--> Persistence Manager
              - save_to_path(path: &Path) -> Result<(), String>
              - load_from_path(path: &Path) -> Result<Self, String>
```

### 4.2 Invariant Rules (CS1..CS5)

- **CS1 (Store Integrity & Active Layout Constraint)**:
  - `store` must always contain at least one valid layout.
  - `active_layout_id` must reference a valid layout present in `store`.
  - Built-in canonical presets (`aios-uefi-standard-v1`, `aios-container-minimal-v1`) cannot be removed or replaced with an invalid spec.
- **CS2 (Target Budget & Feasibility Invariant)**:
  - `probe_target(spec, disk_bytes)` must assert:
    1. `disk_bytes >= spec.target_disk_min_bytes`.
    2. `disk_bytes >= (sum(partition.size_mib) * 1024 * 1024)`.
  - Shortfalls must return a descriptive error and evaluation failure.
- **CS3 (Differential Safety & Destructiveness Invariant)**:
  - `diff_layouts(source, target)` must detect:
    - Added, removed, or resized partitions.
    - Modified mount paths, options, or devices.
    - Added, removed, or changed directories.
  - If a partition is removed or shrunk, or if a mount filesystem type changes, `LayoutDiff.destructive` must evaluate to `true`.
- **CS4 (Fstab Roundtrip & Order Invariant)**:
  - Generating fstab must preserve topological order (`/` before `/boot/efi`).
  - Parsing a generated fstab back into mount specs must match the original mounts without loss of fields.
- **CS5 (Atomic Persistence Invariant)**:
  - Saving to disk must use an atomic write (`.tmp` -> `rename`).
  - Loading from disk must validate all deserialized layouts and active ID before accepting the state.

---

## 5. Unknowns and Decisions Needed

1. **Decision 1: Storage Location for Layout State**:
   - *Options*: `/etc/aios/fs_layouts.json` vs `/var/lib/aios/fs_layouts.json`.
   - *Resolution*: Use `/etc/aios/fs_layouts.json` as the default path since layout profiles represent static system configuration rather than volatile runtime state, while allowing arbitrary path overrides for testing.

2. **Decision 2: Reconciling External fstab with In-Memory Layout**:
   - *Question*: When parsing an existing host `/etc/fstab`, how should pseudo-filesystems (like `cgroup`, `securityfs`) that may not be in the minimal preset be treated?
   - *Resolution*: Treat standard recognized filesystems and custom mounts as valid `MountPointSpec` entries; mark non-essential mounts with `required: false`.

3. **Decision 3: Breaking Changes Flag in Layout Diff**:
   - *Question*: What qualifies as a "destructive" change in `LayoutDiff`?
   - *Resolution*:
     - Deleting an existing partition.
     - Shrinking partition size (`new_size < old_size`).
     - Changing the filesystem type of an existing mount.
     - Removing the root mount `/` or ESP `/boot/efi`.

---

## 6. Citations & Bibliography

1. Linux Foundation. *Filesystem Hierarchy Standard (FHS 3.0)*, March 2015.
2. Kerrisk, Michael. *The Linux Programming Interface: A Linux and UNIX System Programming Handbook*, No Starch Press, 2010 (Chapters 14, 18, 40).
3. Poettering, Lennart. *The Case for the /usr Merge*, freedesktop.org, 2012.
4. UEFI Forum. *Unified Extensible Firmware Interface (UEFI) Specification, Version 2.10*, Aug 2022.
5. Center for Internet Security. *CIS Debian Linux Benchmark v2.0.0*, 2024.
