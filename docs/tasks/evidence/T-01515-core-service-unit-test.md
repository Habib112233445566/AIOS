# T-01515: Filesystem Layout - Core Service: Unit Test

## Metadata
- **Task ID:** `T-01515`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout Core Service (`code/aiosh-rust/aiosh-core::fs_layout_service`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (6/10) — Core Service Unit Test
- **Dependencies:** `T-01514` (Core Service Implementation)
- **Next Task:** `T-01516` (Filesystem Layout / core service: Integration)

---

## 1. Unit Test Suite Architecture

A dedicated automated test battery has been authored in `code/aiosh-rust/aiosh-core/tests/test_fs_layout_service.rs` validating functional correctness, boundary limits, and negative failure modes:

1. **`test_fs_layout_service_scaffold_initialization`**:
   - Asserts default active layout (`aios-uefi-standard-v1`).
   - Asserts built-in canonical presets (`standard_uefi`, `minimal_container`).
   - Asserts sterile `empty()` constructor.
2. **`test_fs_layout_service_store_crud`**:
   - Tests listing all profiles.
   - Tests registering custom layout.
   - Tests duplicate rejection.
   - Tests switching active layout.
   - Tests protection of active layout against deletion.
   - Tests protection of built-in presets against deletion.
   - Tests successful deletion of non-active custom layouts.
3. **`test_fs_layout_service_probe_target`**:
   - Tests viable target disk (100 GiB) with zero errors.
   - Tests undersized disk (20 GiB) rejecting layout with explicit budget deficit errors.
   - Tests non-existent layout lookup.
4. **`test_fs_layout_service_diff_layouts`**:
   - Tests cross-preset comparison between UEFI and container layouts, correctly detecting partition removal and setting `destructive: true`.
   - Tests non-destructive layout enlargement (expanding root from 50 GiB to 60 GiB), verifying `destructive: false` and accurate `partitions_modified` details.
5. **`test_fs_layout_service_fstab_export_and_import`**:
   - Verifies fstab generation containing EFI and root entries.
   - Verifies importing raw fstab into a typed layout spec inheriting base settings.
6. **`test_fs_layout_service_atomic_persistence`**:
   - Verifies writing to temporary path with immediate rename.
   - Verifies roundtrip deserialization and active layout integrity.
7. **`test_fs_layout_service_negative_registration_and_validation`**:
   - Asserts rejection of layout violating FL1 (root pass number != 1).
   - Asserts rejection of layout missing root mount.
8. **`test_fs_layout_service_probe_boundary_and_slack_warnings`**:
   - Tests exact boundary: 1 byte below required disk minimum.
   - Tests tight slack boundary: disk with < 10% free capacity emits diagnostic warning.
9. **`test_fs_layout_service_negative_store_and_diff_operations`**:
   - Asserts error upon removing non-existent layout ID.
   - Asserts error upon setting non-existent active layout ID.
   - Asserts error upon diffing non-existent source or target layout.
10. **`test_fs_layout_service_negative_fstab_and_persistence`**:
    - Asserts rejection of empty fstab content.
    - Asserts error reporting on malformed fstab lines.
    - Asserts error on exporting non-existent layout.
    - Asserts error on loading from non-existent file or corrupted JSON payload.

---

## 2. Test Execution Output

```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_fs_layout_service
   Compiling aiosh-core v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.05s
     Running tests\test_fs_layout_service.rs

running 10 tests
test test_fs_layout_service_fstab_export_and_import ... ok
test test_fs_layout_service_diff_layouts ... ok
test test_fs_layout_service_negative_registration_and_validation ... ok
test test_fs_layout_service_negative_store_and_diff_operations ... ok
test test_fs_layout_service_probe_boundary_and_slack_warnings ... ok
test test_fs_layout_service_probe_target ... ok
test test_fs_layout_service_atomic_persistence ... ok
test test_fs_layout_service_scaffold_initialization ... ok
test test_fs_layout_service_store_crud ... ok
test test_fs_layout_service_negative_fstab_and_persistence ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

Acceptance criteria satisfied:
- New test file runs standalone and passes (10/10 PASS).
- Negative cases, boundary values, and primary failure modes are asserted.
