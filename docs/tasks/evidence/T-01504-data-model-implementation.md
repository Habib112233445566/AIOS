# T-01504: Filesystem Layout - Data Model: Implementation

## Metadata
- **Task ID:** `T-01504`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout (`code/aiosh-rust/aiosh-core::fs_layout`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (4/10) — Data Model Implementation
- **Dependencies:** `T-01503` (Scaffold)
- **Next Task:** `T-01505` (Unit Test)

---

## 1. Executive Summary & Deliverables

Task `T-01504` implemented the full operational data model and validation engine for the Filesystem Layout subsystem in [`code/aiosh-rust/aiosh-core/src/fs_layout.rs`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/src/fs_layout.rs):

1. **`FsType`**: Complete enumeration of standard Linux filesystems (`Ext4`, `Btrfs`, `Xfs`, `Vfat`, `Tmpfs`, `Devtmpfs`, `Procfs`, `Sysfs`, `Overlayfs`, `Squashfs`, `Swap`, `Custom`). Implemented bidirectional conversions via `Display` and `FromStr`.
2. **`PartitionType`**: Partition classification for GPT tables with canonical GUID mappings (`gpt_type_guid()` and `from_gpt_guid()`):
   - `EfiSystem`: `c12a7328-f81f-11d2-ba4b-00a0c93ec93b`
   - `LinuxRoot`: `4f68bce3-e8cd-4db1-96e7-fbcaf984b709`
   - `LinuxHome`: `933ac7e1-2eb4-4f13-b844-0e14e2aef915`
   - `LinuxSwap`: `0657fd6d-a4ab-43c4-84e5-0933c84b4f4f`
   - `LinuxVar`: `4d21b016-b534-45c2-a9fb-5c16e091fd2d`
   - `LinuxGeneric`: `0fc63daf-8483-4772-8e79-3d69d8477de4`
3. **`MountPointSpec`**: Full implementation of `fstab(5)` 6-field formatting (`to_fstab_line`) and parsing (`parse_fstab_line`), handling comments, arbitrary whitespace delimiters, and mount options.
4. **`FilesystemLayoutSpec`**: Complete layout definitions, including JSON round-tripping (`to_json`, `from_json`), `/etc/fstab` file generation (`generate_fstab`), and standard reference presets:
   - `standard_uefi()`: Reference 64 GiB GPT layout with ESP (512 MiB FAT32), Root (50 GiB ext4), Swap (4 GiB), CIS temporary mounts (`/tmp`, `/dev/shm` with `nodev,nosuid,noexec`), and core AIOS directories (`/var/lib/aios`, `/run/aios`, `/etc/aios`, `/var/log/aios`).
   - `minimal_container()`: Reference virtual layout with `/proc`, `/sys`, `/tmp` tmpfs mounts.
5. **Invariant System `FL1..FL5`**:
   - `FL1`: Exactly one root mount point with pass number 1.
   - `FL2`: Path hygiene (absolute paths, no relative `..` traversals, no empty segments, no control characters, no trailing slashes).
   - `FL3`: Mount hierarchy topology (ancestor mounts must precede descendant mounts, no duplicates).
   - `FL4`: CIS benchmark security options (`nodev` and `nosuid` mandatory on `/tmp` and `/dev/shm`).
   - `FL5`: Partition sanity (positive sizes, 1..=128 unique indices, total size bounded by disk capacity, ESP $\ge 100$ MiB formatted as `vfat`).

---

## 2. Test Execution & Output

Executed `cargo test -p aiosh-core --test test_fs_layout_data_model`:
```
running 11 tests
test test_fl2_path_hygiene ... ok
test test_fl3_mount_order_hierarchy ... ok
test test_fl1_single_root_mount_enforcement ... ok
test test_directory_spec_validation ... ok
test test_fl5_partition_constraints ... ok
test test_fstab_serialization_and_parsing ... ok
test test_fl4_cis_security_options ... ok
test test_layout_json_roundtrip ... ok
test test_partition_type_gpt_guid_mapping ... ok
test test_minimal_container_layout_validity ... ok
test test_standard_uefi_layout_validity ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

All acceptance criteria satisfied:
- [x] Targeted test passes.
- [x] No regressions in existing suites (`aiosh-core --lib` passed 372/372 tests).
