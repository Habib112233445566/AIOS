# T-01518: Filesystem Layout - Core Service: Hardening

## Metadata
- **Task ID:** `T-01518`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout Core Service (`code/aiosh-rust/aiosh-core::fs_layout_service`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (9/10) — Core Service Hardening
- **Dependencies:** `T-01517` (Core Service Security Review)
- **Next Task:** `T-01519` (Filesystem Layout / core service: Documentation)

---

## 1. Hardening Measures Implemented

To protect the filesystem layout service against resource exhaustion, malformed payloads, and silent failures, the following hardening controls were implemented:

1. **Size Caps & Memory Limits**:
   - `load_from_path`: Added strict 10 MiB limit on store files using `fs::metadata(path).len() <= 10 * 1024 * 1024` prior to memory reading, preventing memory starvation attacks.
   - `import_fstab_as_layout`: Capped parsed mounts to a maximum of 128 entries (`if parsed_mounts.len() > 128`), matching the partition and mount cardinality invariants from the data model.
2. **Arithmetic & Overflow Safety**:
   - `probe_target`: Refactored partition summation and byte calculation to use `saturating_add` and `saturating_mul(1024 * 1024)`, eliminating integer overflow vulnerabilities with large capacities.
3. **Path Sanitization & Symlink Purge**:
   - `save_to_path`: Enforced path hygiene checks rejecting empty paths or paths containing null bytes (`\0`) or ASCII control characters.
   - Before writing to temporary files (`.tmp.<pid>`), pre-existing files or symlinks at the temporary path are explicitly unlinked (`let _ = fs::remove_file(&tmp_path);`), preventing symlink overwrite hijacks.
4. **Standard Result Envelopes & Honest Audit**:
   - CLI errors are reported with structured exit codes (`0` = success, `1` = operational/validation failure, `2` = argument error) and standard JSON error structures.
   - All state mutations and failed evaluations emit structured audit events to the SQLite WAL audit ring.
   - On persistence failure, temporary files are reliably cleaned up on error paths without resource leaks.

---

## 2. Automated Hardening Test Results

```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_fs_layout_service
   Compiling aiosh-core v0.1.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.75s
     Running tests\test_fs_layout_service.rs

running 11 tests
test test_fs_layout_service_diff_layouts ... ok
test test_fs_layout_service_fstab_export_and_import ... ok
test test_fs_layout_service_atomic_persistence ... ok
test test_fs_layout_service_hardening_mount_caps_and_path_hygiene ... ok
test test_fs_layout_service_negative_registration_and_validation ... ok
test test_fs_layout_service_negative_store_and_diff_operations ... ok
test test_fs_layout_service_negative_fstab_and_persistence ... ok
test test_fs_layout_service_probe_boundary_and_slack_warnings ... ok
test test_fs_layout_service_probe_target ... ok
test test_fs_layout_service_scaffold_initialization ... ok
test test_fs_layout_service_store_crud ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

Acceptance criteria satisfied:
- Failure modes produce explicit, auditable errors.
- No temp/connection leaks on error paths.
