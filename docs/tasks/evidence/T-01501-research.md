# T-01501: Filesystem Layout - Data Model: Research

## Metadata
- **Task ID:** `T-01501`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout (`code/aiosh-rust/aiosh-core::fs_layout`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic Launch: Filesystem Layout (1/10) — Data Model Research

---

## 1. Executive Overview & Mission Context

In the AIOS architecture, following User Session Bootstrap (`T-01401..T-01500`), the **Filesystem Layout** subsystem defines, validates, configures, and monitors the storage partition topology, directory tree hierarchy, mount configurations (`/etc/fstab`), and storage security boundaries for bootable target environments.

The operational vision for AIOS unites three pillars:
1. **Pillar A (Ethical Hacking on the Inside)**: Underlying Kali Linux Rolling distribution with access to penetration testing tools, raw storage inspection utilities, and kernel capabilities.
2. **Pillar B (Windows-Look Desktop on the Outside)**: Highly accessible graphical desktop environment (XFCE with `kali-undercover` or KDE Plasma Fluent theme) delivered on top of a standardized, stable filesystem hierarchy.
3. **Pillar C (S-Rank AI Kernel Subsystem)**: An autonomous AI shell operating as a desktop companion and daemon, maintaining persistent state in `/var/lib/aios`, volatile runtime sockets in `/run/aios`, configuration in `/etc/aios`, and tamper-evident audit logs in `/var/log/aios`.

The Filesystem Layout data model must canonically represent and govern:
- Disk partitioning schemes (GPT partition tables, ESP, Root, Home, Swap).
- Supported filesystem types (`ext4`, `btrfs`, `xfs`, `vfat`, `tmpfs`, `procfs`, `sysfs`, `devtmpfs`, `overlayfs`, `squashfs`).
- Mount point specifications (`/`, `/boot`, `/boot/efi`, `/var`, `/tmp`, `/run`, `/home`) with standard 6-field `fstab` semantics (`device`, `mount_point`, `fs_type`, `options`, `dump`, `pass`).
- Modern UsrMerge conventions (`/bin -> /usr/bin`, `/sbin -> /usr/sbin`, `/lib -> /usr/lib`).
- Standard directory permissions and ownerships (FHS 3.0 compliance, sticky bit `/tmp`, restrictive `0700` user runtimes).
- Mount security options enforcing CIS benchmarks (`nodev`, `nosuid`, `noexec` on `/tmp` and `/dev/shm`).

---

## 2. Authoritative Sources & Upstream Standards

1. **Filesystem Hierarchy Standard (FHS 3.0)**:
   - *Source*: Linux Foundation FHS 3.0 Specification (2015).
   - Core root directories: `/`, `/boot`, `/dev`, `/etc`, `/home`, `/lib`, `/media`, `/mnt`, `/opt`, `/proc`, `/root`, `/run`, `/sbin`, `/srv`, `/sys`, `/tmp`, `/usr`, `/var`.
   - Distinguishes shareable vs. unshareable and variable vs. static files.
   - Requires `/tmp` to support temporary file creation and `/var/tmp` for persistent temporary files.

2. **freedesktop.org & Debian UsrMerge Specification**:
   - *Source*: Debian Wiki UsrMerge, systemd TheCaseForTheUsrMerge.
   - All binaries reside in `/usr/bin` with `/bin` and `/sbin` as compatibility symlinks.
   - All system libraries reside in `/usr/lib` with `/lib` and `/lib64` as compatibility symlinks.
   - Simplifies atomic OS snapshots, vendor package delivery, and read-only `/usr` mounting.

3. **UEFI Specification v2.10 & DISK GPT GUIDs**:
   - *Source*: Unified Extensible Firmware Interface (UEFI) Specification, Section 13 (Protocols - Media Access).
   - EFI System Partition (ESP): FAT32 (`vfat`) filesystem, GPT Partition Type GUID `c12a7328-f81f-11d2-ba4b-00a0c93ec93b`, mounted at `/boot/efi` with restrictive permissions (`umask=0077`).
   - Linux Root x86_64: GUID `4f68bce3-e8cd-4db1-96e7-fbcaf984b709`.
   - Linux Swap: GUID `0657fd6d-a4ab-43c4-84e5-0933c84b4f4f`.
   - Linux Home: GUID `933ac7e1-2eb4-4f13-b844-0e14e2aef915`.

4. **Linux `fstab(5)` and `mount(8)` Semantics**:
   - *Source*: Linux Programmer's Manual `fstab(5)`, `mount(8)`, `libmount`.
   - Entry format: `fs_spec  fs_file  fs_vfstype  fs_mntops  fs_freq  fs_passno`.
   - Mount options: `rw`/`ro`, `suid`/`nosuid`, `dev`/`nodev`, `exec`/`noexec`, `auto`/`noauto`, `async`/`sync`, `relatime`/`noatime`.
   - Pass numbers: 1 for root `/`, 2 for other on-disk filesystems, 0 for virtual/pseudo filesystems (`tmpfs`, `proc`).

5. **CIS Debian/Linux Benchmark (v2.0+) & DISA STIG**:
   - Rule 1.1.2..1.1.5: Ensure `/tmp` is a separate mount with `nodev`, `nosuid`, and `noexec`.
   - Rule 1.1.6..1.1.9: Ensure `/dev/shm` is mounted with `nodev`, `nosuid`, and `noexec`.
   - Rule 1.1.10..1.1.13: Ensure separate partitions for `/var` and `/var/log` to prevent log exhaustion denial of service.

---

## 3. Facts vs. Assumptions

| Fact | Assumption |
|---|---|
| A Linux system cannot boot without a root filesystem (`/`) and essential virtual filesystems (`/proc`, `/sys`, `/dev`). | AIOS can represent both bare-metal disk partitioning and virtual/container/Live mount schemes in a unified data model. |
| UEFI systems require an ESP formatted as FAT32 (`vfat`) with unique GPT partition type GUID. | In non-UEFI or container test environments, ESP partition requirements can be designated optional or virtualized. |
| `fstab` requires 6 distinct whitespace-separated fields per line, with pass numbers determining fsck ordering. | AIOS can serialize and parse `fstab` lines losslessly into typed Rust structs. |
| Modern Debian and Kali Linux enforce UsrMerge (`/bin -> usr/bin`, etc.). | The AIOS filesystem data model should model UsrMerge compatibility links as first-class assertions. |
| AIOS components require dedicated operational paths (`/var/lib/aios`, `/run/aios`, `/var/log/aios`, `/etc/aios`). | Validating and provisioning these standard AIOS directories during layout initialization prevents daemon startup failures. |

---

## 4. Proposed Data Model & Invariants

### 4.1 Proposed Rust Types (`code/aiosh-rust/aiosh-core/src/fs_layout.rs`)

1. **`FsType`**: Enum
   - `Ext4`, `Btrfs`, `Xfs`, `Vfat`, `Tmpfs`, `Devtmpfs`, `Procfs`, `Sysfs`, `Overlayfs`, `Squashfs`, `Swap`, `Custom(String)`.

2. **`PartitionType`**: Enum
   - `EfiSystem` (ESP), `LinuxRoot`, `LinuxHome`, `LinuxSwap`, `LinuxVar`, `LinuxGeneric`.

3. **`MountPointSpec`**: Struct
   - `path`: Normalized absolute mount point (e.g., `/`, `/boot/efi`, `/var`).
   - `device`: Device specifier, UUID (`UUID=...`), LABEL (`LABEL=...`), or virtual name (`tmpfs`, `proc`).
   - `fs_type`: `FsType`.
   - `options`: List of mount option strings (`["rw", "relatime", "nodev", "nosuid"]`).
   - `dump`: u32 (usually 0 or 1).
   - `pass`: u32 (0 for pseudo, 1 for root, 2 for other filesystems).
   - `required`: bool.

4. **`PartitionSpec`**: Struct
   - `index`: u32.
   - `label`: String.
   - `partition_type`: `PartitionType`.
   - `size_mb`: Option<u64> (None indicates filling remaining space).
   - `fs_type`: `FsType`.
   - `mount_point`: Option<String>.
   - `uuid`: Option<String>.

5. **`DirectorySpec`**: Struct
   - `path`: Normalized absolute path.
   - `owner_uid`: u32.
   - `group_gid`: u32.
   - `mode`: u32 (POSIX permission bits e.g. `0o755`, `0o1777`).
   - `essential`: bool.

6. **`FilesystemLayoutSpec`**: Top-Level Store Struct
   - `name`: String (e.g. `kali-default-uefi`).
   - `description`: String.
   - `partitions`: Vec<PartitionSpec>.
   - `mounts`: Vec<MountPointSpec>.
   - `directories`: Vec<DirectorySpec>.
   - `created_at`: String (RFC-3339).

### 4.2 Invariant System (`FL1..FL5`)

1. **`FL1` (Root Mount Invariant)**:
   Every valid `FilesystemLayoutSpec` must contain exactly one mount point for `/` (`pass == 1`, `required == true`).
2. **`FL2` (Path Normalization & Hygiene)**:
   All mount points and directory paths must be absolute, normalized, without trailing slashes (except `/`), without traversal elements (`..`, `.`), and free of control characters.
3. **`FL3` (Mount Hierarchy Topology)**:
   Mount points must not contain duplicate targets, and parent mounts must precede child mounts (e.g. `/` precedes `/boot`, which precedes `/boot/efi`).
4. **`FL4` (Security Hardening Mount Options)**:
   Volatile user-accessible filesystems (`/tmp`, `/dev/shm`) must include `nosuid` and `nodev` in their mount options.
5. **`FL5` (Capacity & Size Bounds)**:
   Partition sizes must be strictly positive (> 0 MB), and total partition entries per layout must not exceed `MAX_PARTITIONS = 128`.

---

## 5. Unknowns & Decisions Needed Before Implementation

1. **Decision 1: Storage Target Scope for Live / Bootable Target vs Installed System**
   - *Question*: Should `FilesystemLayoutSpec` distinguish between a Live ISO/squashfs layout and an installed disk layout?
   - *Decision*: Yes. Add an optional `layout_type: FsLayoutType` (`Installed`, `LiveBoot`, `Container`, `Virtual`) with appropriate default mount profiles.
2. **Decision 2: Symlink Tracking in Directory Model**
   - *Question*: How should UsrMerge symlinks (`/bin -> usr/bin`) be represented?
   - *Decision*: Add a `symlinks: Vec<SymlinkSpec>` to `FilesystemLayoutSpec` specifying `link_path` and `target_path`.
3. **Decision 3: Fstab Serialization Format**
   - *Question*: Should fstab export be rendered directly from `MountPointSpec`?
   - *Decision*: Yes. Provide a `to_fstab()` method that generates clean, standard tabular `/etc/fstab` text with column alignment.

---

## 6. Conclusion

The data model for Filesystem Layout can be cleanly implemented in `code/aiosh-rust/aiosh-core/src/fs_layout.rs` following the proven patterns of `session.rs` and `service.rs`. No external crates are required; standard Rust library collections, `chrono`, and `serde` are sufficient.
