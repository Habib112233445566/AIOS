# T-01505: Filesystem Layout - Data Model: Unit Test

## Metadata
- **Task ID:** `T-01505`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout (`code/aiosh-rust/aiosh-core::fs_layout`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (5/10) — Data Model Unit Test
- **Dependencies:** `T-01504` (Implementation)
- **Next Task:** `T-01506` (Integration)

---

## 1. Executive Summary & Test Suite Structure

Task `T-01505` established a dedicated automated unit test suite in [`code/aiosh-rust/aiosh-core/tests/test_fs_layout_data_model.rs`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/tests/test_fs_layout_data_model.rs). The suite exercises both happy paths and negative failure modes across the entire data model and invariant rules (`FL1..FL5`).

### 1.1 Test Matrix (19 Tests)

| Category | Test Function | Verified Contract / Invariant |
|---|---|---|
| **Happy Path** | `test_standard_uefi_layout_validity` | Validates reference 64 GiB UEFI GPT layout |
| **Happy Path** | `test_minimal_container_layout_validity` | Validates minimal container/chroot virtual mounts |
| **Happy Path** | `test_layout_json_roundtrip` | Verifies serde round-tripping for `FilesystemLayoutSpec` |
| **Happy Path** | `test_partition_type_gpt_guid_mapping` | Verifies GPT GUID resolution and string mapping |
| **Happy Path** | `test_fs_type_roundtrip_and_custom` | Verifies parsing and Display of standard and custom FSTypes |
| **FL1: Root Mount** | `test_fl1_single_root_mount_enforcement` | Rejects missing root or duplicate root mounts |
| **FL1: Root Pass** | `test_fl1_root_pass_number_validation` | Enforces pass=1 for root, rejects pass=1 for non-root |
| **FL2: Path Hygiene** | `test_fl2_path_hygiene` | Rejects relative paths, `..` traversals, and trailing slashes |
| **FL2: Path Hygiene** | `test_fl2_path_control_chars_and_oversized` | Rejects null bytes, newlines, empty components (`//`), and paths > 1024 chars |
| **FL3: Topology** | `test_fl3_duplicate_mount_paths` | Rejects multiple mount entries for the same target directory |
| **FL3: Topology** | `test_fl3_mount_order_hierarchy` | Enforces parent mount preceding child mounts (e.g. `/` before `/boot/efi`) |
| **FL4: CIS Security** | `test_fl4_cis_security_options` | Enforces `nodev` and `nosuid` on `/tmp` |
| **FL4: CIS Security** | `test_fl4_dev_shm_cis_options` | Enforces `nodev` and `nosuid` on `/dev/shm` |
| **FL5: Partitions** | `test_fl5_partition_constraints` | Rejects partition index 0, non-vfat ESP, and zero-sized partitions |
| **FL5: Partitions** | `test_fl5_total_partition_budget_exceeded` | Rejects partition totals exceeding `target_disk_min_bytes` |
| **FL5: Partitions** | `test_fl5_esp_mount_filesystem_mismatch` | Enforces `/boot/efi` mount using `vfat` when ESP partition exists |
| **Directories** | `test_directory_spec_validation` | Verifies directory permissions, owner, group, and UsrMerge links |
| **Fstab Formatting** | `test_fstab_serialization_and_parsing` | Validates standard 6-field generation and lossless line parsing |
| **Fstab Parsing** | `test_fstab_malformed_lines` | Rejects lines with < 4 fields, > 6 fields, and malformed numeric fields |

---

## 2. Test Execution Output

```
$ cargo test -p aiosh-core --test test_fs_layout_data_model
   Compiling aiosh-core v0.1.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.46s
     Running tests\test_fs_layout_data_model.rs

running 19 tests
test test_directory_spec_validation ... ok
test test_fl2_path_control_chars_and_oversized ... ok
test test_fl1_single_root_mount_enforcement ... ok
test test_fl1_root_pass_number_validation ... ok
test test_fl3_duplicate_mount_paths ... ok
test test_fl2_path_hygiene ... ok
test test_fl3_mount_order_hierarchy ... ok
test test_fl4_cis_security_options ... ok
test test_fl4_dev_shm_cis_options ... ok
test test_fl5_partition_constraints ... ok
test test_fl5_esp_mount_filesystem_mismatch ... ok
test test_fs_type_roundtrip_and_custom ... ok
test test_fl5_total_partition_budget_exceeded ... ok
test test_fstab_malformed_lines ... ok
test test_fstab_serialization_and_parsing ... ok
test test_minimal_container_layout_validity ... ok
test test_partition_type_gpt_guid_mapping ... ok
test test_standard_uefi_layout_validity ... ok
test test_layout_json_roundtrip ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

All acceptance criteria satisfied:
- [x] New test file runs standalone and passes (19/19 tests ok).
- [x] Negative cases and boundary conditions are rigorously asserted.
