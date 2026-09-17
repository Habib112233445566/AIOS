# T-01514: Filesystem Layout - Core Service: Implementation

## Metadata
- **Task ID:** `T-01514`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout Core Service (`code/aiosh-rust/aiosh-core::fs_layout_service`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (5/10) — Core Service Implementation
- **Dependencies:** `T-01513` (Core Service Scaffold)
- **Next Task:** `T-01515` (Filesystem Layout / core service: Unit Test)

---

## 1. Summary of Changes

In accordance with specification `T-01512-spec.md`, the full core service logic was implemented in `code/aiosh-rust/aiosh-core/src/fs_layout_service.rs`:

1. **`FilesystemLayoutStore` Implementation**:
   - `new()`: Seeds store with canonical `standard_uefi` and `minimal_container` profiles; defaults `active_layout_id` to `"aios-uefi-standard-v1"`.
   - `empty()`: Initializes clean store for testing.
   - `register_layout()`: Validates layout against `FL1..FL5` before insertion; rejects duplicate layout identifiers; automatically sets active ID if store was empty.
   - `get_layout()` & `list_layouts()`: Read-only accessors.
   - `remove_layout()`: Protects currently active layout from deletion; protects built-in canonical presets from accidental deletion.
   - `get_active_layout()` & `set_active_layout()`: Manages pointer to currently active system layout.

2. **`FilesystemLayoutService` Implementation**:
   - `probe_target(layout_id, target_disk_bytes)`: Computes partition budget sum; verifies disk capacity meets `target_disk_min_bytes` and sum of partitions; generates warnings for tight headroom (< 10% slack).
   - `diff_layouts(source_id, target_id)`: Itemizes partition, mount, and directory changes; evaluates whether changes are destructive (partition removed, partition shrunk, mount filesystem type changed, or root device changed).
   - `export_fstab(layout_id)`: Synthesizes clean `/etc/fstab` text with comments, alignment, and pass numbers.
   - `import_fstab_as_layout(id, name, fstab_content, base_layout_id)`: Parses external fstab text into structured mounts; inherits partitions/directories from base layout; registers resulting layout profile.
   - `save_to_path(path)` & `load_from_path(path)`: Implements atomic file persistence using sibling temporary files (`.tmp.<pid>`) with immediate rename, preventing partial writes during crashes; validates loaded structures on deserialization.

---

## 2. Test Execution & Verification

Executed targeted test suite covering store operations, disk probing, diffing, fstab import/export, and persistence:

```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_fs_layout_service
   Compiling aiosh-core v0.1.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.01s
     Running tests\test_fs_layout_service.rs

running 6 tests
test test_fs_layout_service_probe_target ... ok
test test_fs_layout_service_fstab_export_and_import ... ok
test test_fs_layout_service_diff_layouts ... ok
test test_fs_layout_service_scaffold_initialization ... ok
test test_fs_layout_service_store_crud ... ok
test test_fs_layout_service_atomic_persistence ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Regression test on data model:
```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_fs_layout_data_model
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Acceptance criteria satisfied:
- Targeted tests pass.
- No regression in existing smoke suites for touched modules.
