# T-01502: Filesystem Layout - Data Model: Specification

## Metadata
- **Task ID:** `T-01502`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout (`code/aiosh-rust/aiosh-core::fs_layout`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (2/10) — Data Model Specification
- **Dependencies:** `T-01501` (Research)
- **Next Task:** `T-01503` (Scaffold)

---

## 1. Executive Summary & Purpose

The **Filesystem Layout** subsystem provides a typed, canonical data model and verification engine governing:
1. Disk partitioning topology (GPT partition definitions, ESP, root, home, swap).
2. Filesystem type classification (`ext4`, `btrfs`, `xfs`, `vfat`, `tmpfs`, `devtmpfs`, `procfs`, `sysfs`, `overlayfs`, `squashfs`, `swap`).
3. Standard Linux 6-field `fstab(5)` serialization and parsing (`device`, `mount_point`, `fs_type`, `options`, `dump`, `pass`).
4. Mount hierarchy ordering and CIS security option verification (`nodev`, `nosuid`, `noexec`).
5. Filesystem Hierarchy Standard (FHS 3.0) directory structures, UsrMerge compatibility symlinks, and AIOS operational runtime paths (`/var/lib/aios`, `/run/aios`, `/etc/aios`, `/var/log/aios`).

This specification defines the exact Rust types, serialization formats, validation invariants, error codes, and audit effects without introducing undeclared third-party dependencies.

---

## 2. Reused vs. New Interfaces

### 2.1 Reused Existing Interfaces
- `serde::{Serialize, Deserialize}`: Standard JSON serialization and deserialization across AIOS rust crates.
- `std::fmt::{Display, Formatter}`: Standard Rust formatting for string representations.
- `std::path::{Path, PathBuf, Component}`: Safe path parsing and component analysis.
- `aiosh-core::audit`: Downstream service integration for audit event logging.

### 2.2 New Interfaces (AIOS-Specific)
- Module `aiosh_core::fs_layout`:
  - Enums: `FsType`, `PartitionType`.
  - Structs: `MountPointSpec`, `PartitionSpec`, `DirectorySpec`, `FilesystemLayoutSpec`.
  - Invariant validator: `validate_filesystem_layout`, `validate_mount_point`, `validate_partition_spec`, `validate_directory_spec`.
  - Serialization / Parsing utilities: `MountPointSpec::to_fstab_line`, `MountPointSpec::parse_fstab_line`, `FilesystemLayoutSpec::generate_fstab`.

---

## 3. Data Model Specification

### 3.1 Supported Filesystem Types: `FsType`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FsType {
    Ext4,
    Btrfs,
    Xfs,
    Vfat,
    Tmpfs,
    Devtmpfs,
    Procfs,
    Sysfs,
    Overlayfs,
    Squashfs,
    Swap,
    Custom(String),
}
```

#### String Mapping (Fstab and Kernel Driver Names)
- `Ext4` $\leftrightarrow$ `"ext4"`
- `Btrfs` $\leftrightarrow$ `"btrfs"`
- `Xfs` $\leftrightarrow$ `"xfs"`
- `Vfat` $\leftrightarrow$ `"vfat"`
- `Tmpfs` $\leftrightarrow$ `"tmpfs"`
- `Devtmpfs` $\leftrightarrow$ `"devtmpfs"`
- `Procfs` $\leftrightarrow$ `"proc"`
- `Sysfs` $\leftrightarrow$ `"sysfs"`
- `Overlayfs` $\leftrightarrow$ `"overlay"`
- `Squashfs` $\leftrightarrow$ `"squashfs"`
- `Swap` $\leftrightarrow$ `"swap"`
- `Custom(s)` $\leftrightarrow$ `s`

### 3.2 Partition Types: `PartitionType`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartitionType {
    /// EFI System Partition (ESP). GPT GUID: c12a7328-f81f-11d2-ba4b-00a0c93ec93b
    EfiSystem,
    /// Linux x86-64 Root filesystem. GPT GUID: 4f68bce3-e8cd-4db1-96e7-fbcaf984b709
    LinuxRoot,
    /// Linux /home filesystem. GPT GUID: 933ac7e1-2eb4-4f13-b844-0e14e2aef915
    LinuxHome,
    /// Linux Swap partition. GPT GUID: 0657fd6d-a4ab-43c4-84e5-0933c84b4f4f
    LinuxSwap,
    /// Linux /var partition. GPT GUID: 4d21b016-b534-45c2-a9fb-5c16e091fd2d
    LinuxVar,
    /// Generic Linux partition (0x8300 / 0FC63DAF-8483-4772-8E79-3D69D8477DE4)
    LinuxGeneric,
    /// Custom partition type GUID
    Custom(String),
}
```

### 3.3 Mount Point Specification: `MountPointSpec`

Represents one entry in `/etc/fstab` (6-field semantics):

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MountPointSpec {
    /// Mount point directory (e.g. "/", "/boot/efi", "/tmp").
    pub path: String,
    /// Device identifier: UUID="...", LABEL="...", /dev/..., or pseudo-name ("tmpfs", "proc").
    pub device: String,
    /// Filesystem driver type.
    pub fs_type: FsType,
    /// Mount options (e.g. ["defaults", "rw", "nodev", "nosuid"]).
    pub options: Vec<String>,
    /// Dump frequency (0 = do not dump, 1 = backup).
    pub dump: u32,
    /// fsck pass order (0 = do not check, 1 = root fs, 2 = all other fs).
    pub pass: u32,
    /// Whether this mount is strictly required for operational readiness.
    pub required: bool,
}
```

### 3.4 Partition Specification: `PartitionSpec`

Represents a single partition on a target storage block device:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionSpec {
    /// Partition table slot index (1-based, e.g. 1..128).
    pub index: u32,
    /// Human-readable partition label (e.g. "EFI", "AIOS_ROOT").
    pub label: String,
    /// Partition type classification.
    pub partition_type: PartitionType,
    /// Partition capacity in Megabytes (MiB).
    pub size_mib: u64,
    /// UUID of the partition if pre-assigned.
    pub uuid: Option<String>,
    /// Whether the bootable / legacy-BIOS boot flag is set.
    pub bootable: bool,
    /// Optional default filesystem to format onto the partition.
    pub format_as: Option<FsType>,
}
```

### 3.5 Directory Specification: `DirectorySpec`

Represents a directory in the filesystem tree, including permissions, ownership, and UsrMerge symlinks:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectorySpec {
    /// Normalized absolute directory path (e.g. "/var/lib/aios", "/tmp").
    pub path: String,
    /// POSIX permissions mode in octal (e.g. 0o755, 0o700, 0o1777).
    pub mode: u32,
    /// User ownership name (e.g. "root", "aios").
    pub owner: String,
    /// Group ownership name (e.g. "root", "aios").
    pub group: String,
    /// Functional purpose / description.
    pub description: String,
    /// If this path is a compatibility symlink (UsrMerge), target relative path (e.g. "usr/bin").
    pub symlink_target: Option<String>,
}
```

### 3.6 Complete Filesystem Layout: `FilesystemLayoutSpec`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesystemLayoutSpec {
    /// Unique identifier for the layout (e.g. "aios-uefi-standard-v1").
    pub id: String,
    /// Display name.
    pub name: String,
    /// Detailed description of the target system architecture.
    pub description: String,
    /// Minimum block device capacity required to house this layout in bytes.
    pub target_disk_min_bytes: u64,
    /// Ordered list of partitions to create on the storage disk.
    pub partitions: Vec<PartitionSpec>,
    /// Mount specifications to write to /etc/fstab and verify during boot.
    pub mounts: Vec<MountPointSpec>,
    /// Essential directories to provision and verify for FHS 3.0 and AIOS.
    pub directories: Vec<DirectorySpec>,
    /// UTC timestamp of layout creation.
    pub created_at: String,
}
```

---

## 4. Invariant Rules (FL1..FL5)

Every instance of `FilesystemLayoutSpec` must pass `validate_filesystem_layout(&self) -> Result<(), String>` which enforces the following invariants:

### FL1: Exactly One Root Mount (`/`)
- Exactly one entry in `mounts` must have `path == "/"`.
- The root mount must have `pass == 1`.
- All non-root mounts must have `pass == 0` (for virtual/network filesystems) or `pass == 2` (for local partitions).
- The root mount cannot be marked `required: false`.

### FL2: Path Hygiene & Normalization
- All paths (`mounts[].path`, `directories[].path`) must be non-empty and start with `/`.
- Paths cannot contain `.` or `..` path segments (traversal prevention).
- Paths cannot contain double slashes `//`.
- Paths cannot contain ASCII control characters (`\0`..`\x1F`, `\x7F`).
- Paths cannot have trailing slashes, with the sole exception of the root mount point `"/"`.
- Maximum path length is capped at 1,024 characters.

### FL3: Mount Hierarchy Topology
- Mount points must not contain duplicate paths.
- Mounts must be ordered such that a parent directory precedes its children.
  - Specifically: if mount point $A$ is an ancestor of mount point $B$ (e.g. `/` is ancestor of `/boot`, `/boot` is ancestor of `/boot/efi`), $A$ must appear before $B$ in the mount list to ensure proper mount mounting sequence.

### FL4: CIS Security Mount Options
- For temporary mount points `/tmp` and `/dev/shm`:
  - `options` MUST contain `"nodev"` and `"nosuid"`.
  - When mounted as `tmpfs`, `options` SHOULD include `"noexec"` where compatible.
- `/boot/efi` ESP mount options must enforce restrictive umask (e.g. `umask=0077`).

### FL5: Partition Constraints & ESP Alignment
- Partition indices must be positive ($1 \le \text{index} \le 128$) and strictly unique.
- Partition sizes must be positive ($\text{size_mib} > 0$).
- Total partition capacity must not exceed `target_disk_min_bytes / (1024 * 1024)`.
- If an `EfiSystem` partition is present:
  - Its size must be at least 100 MiB (standard ESP minimum).
  - Its format (if specified) must be `FsType::Vfat`.
  - A corresponding mount point for `/boot/efi` with `FsType::Vfat` should be present in `mounts`.

---

## 5. Operations & Fstab Contract

### 5.1 Fstab Serialization
`MountPointSpec::to_fstab_line(&self) -> String` produces a line conforming to `fstab(5)`:
```
<fs_spec>  <fs_file>  <fs_vfstype>  <fs_mntops>  <fs_freq>  <fs_passno>
```
Example:
```
UUID=a1b2c3d4-e5f6-7890-1234-56789abcdef0  /  ext4  rw,relatime,errors=remount-ro  0  1
```
Options list is serialized as a comma-separated string (`options.join(",")`). If empty, defaults to `"defaults"`.

### 5.2 Fstab Deserialization
`MountPointSpec::parse_fstab_line(line: &str) -> Result<Self, String>`:
- Ignores leading whitespace and lines starting with `#` (returns `None` or handles as comment).
- Splits line by arbitrary whitespace into tokens.
- Fails if fewer than 4 tokens or more than 6 tokens are present:
  - Field 1: device (`fs_spec`).
  - Field 2: mount point (`fs_file`).
  - Field 3: filesystem type (`fs_vfstype`).
  - Field 4: mount options (`fs_mntops`), split by `,`.
  - Field 5: dump (`fs_freq`), optional, defaults to 0.
  - Field 6: pass (`fs_passno`), optional, defaults to 0.
- Normalizes path and maps filesystem string to `FsType`.

---

## 6. Standard Canonical Presets

### 6.1 `FilesystemLayoutSpec::standard_uefi()`
Canonical layout for an AIOS bare-metal or VM installation on a 64 GiB storage device:
1. Partition 1: `ESP` (512 MiB, `PartitionType::EfiSystem`, formatted as `vfat`, mounted at `/boot/efi`).
2. Partition 2: `AIOS_ROOT` (50 GiB, `PartitionType::LinuxRoot`, formatted as `ext4`, mounted at `/`).
3. Partition 3: `AIOS_SWAP` (4 GiB, `PartitionType::LinuxSwap`, `FsType::Swap`).
4. Mounts:
   - `/` (`ext4`, `["rw", "relatime", "errors=remount-ro"]`, pass 1)
   - `/boot/efi` (`vfat`, `["rw", "relatime", "fmask=0077", "dmask=0077"]`, pass 2)
   - `/tmp` (`tmpfs`, `["rw", "nosuid", "nodev", "noexec", "size=4G"]`, pass 0)
   - `/dev/shm` (`tmpfs`, `["rw", "nosuid", "nodev", "noexec"]`, pass 0)
   - `/proc` (`proc`, `["rw", "nosuid", "nodev", "noexec"]`, pass 0)
   - `/sys` (`sysfs`, `["rw", "nosuid", "nodev", "noexec"]`, pass 0)
5. Core AIOS Directories:
   - `/var/lib/aios` (mode `0750`, owner `root`, group `aios`, persistent state)
   - `/run/aios` (mode `0750`, owner `root`, group `aios`, runtime IPC sockets)
   - `/etc/aios` (mode `0755`, owner `root`, group `root`, daemon configuration)
   - `/var/log/aios` (mode `0750`, owner `root`, group `aios`, tamper-evident audit logs)
   - `/tmp` (mode `01777`, sticky bit)
   - `/bin -> usr/bin`, `/sbin -> usr/sbin`, `/lib -> usr/lib` (UsrMerge links)

---

## 7. Error Handling & Failure Paths

All validation failures return explicit error strings structured as:
`"FL<N> violation: <detail>"`

| Code | Trigger | Example Error Message |
|---|---|---|
| `FL1` | Missing or multiple root mounts | `"FL1 violation: layout must have exactly one root ('/') mount, found 0"` |
| `FL1` | Invalid root pass number | `"FL1 violation: root mount pass number must be 1, found 0"` |
| `FL2` | Relative path or traversal | `"FL2 violation: mount path '../etc' contains traversal or is not absolute"` |
| `FL2` | Trailing slash | `"FL2 violation: mount path '/var/' has forbidden trailing slash"` |
| `FL3` | Duplicate mount path | `"FL3 violation: duplicate mount path '/var'"` |
| `FL3` | Inverted mount order | `"FL3 violation: child mount '/boot/efi' appears before parent '/boot'"` |
| `FL4` | Missing security options | `"FL4 violation: mount '/tmp' missing mandatory security option 'nodev'"` |
| `FL5` | ESP formatting mismatch | `"FL5 violation: ESP partition must use vfat filesystem, found ext4"` |
| `FL5` | Total size exceeds budget | `"FL5 violation: total partition size (70000 MiB) exceeds disk budget (65536 MiB)"` |

---

## 8. Audit Integration Contract

Consequential mutations involving filesystem layout operations (e.g. formatting partitions, writing `/etc/fstab`, provisioning AIOS system directories) must emit an audit row via `aiosh-core::audit`:
- **Subsystem**: `"fs_layout"`
- **Actions**:
  - `"fs_layout.validate"` (read-only verification)
  - `"fs_layout.generate_fstab"` (generation of fstab configuration)
  - `"fs_layout.apply"` (disk partitioning and directory creation)
- **Status**: `"ok"` or `"failed"` with explicit error details.
