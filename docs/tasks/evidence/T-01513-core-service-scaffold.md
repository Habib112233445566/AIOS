# T-01513: Filesystem Layout - Core Service: Scaffold

## Metadata
- **Task ID:** `T-01513`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout Core Service (`code/aiosh-rust/aiosh-core::fs_layout_service`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (4/10) — Core Service Scaffold
- **Dependencies:** `T-01512` (Core Service Specification)
- **Next Task:** `T-01514` (Filesystem Layout / core service: Implementation)

---

## 1. Summary of Scaffold Delivered

In accordance with specification `T-01512-spec.md`, the module skeleton for `fs_layout_service` was created and integrated:
1. **Module Creation**: `code/aiosh-rust/aiosh-core/src/fs_layout_service.rs`
   - Defines `FilesystemLayoutStore`, `FilesystemLayoutService`, `TargetEvaluation`, `LayoutDiff`, `PartitionDiffItem`, and `MountDiffItem`.
   - Function signatures implemented with explicit types; unimplemented bodies fail loudly via `unimplemented!()`.
2. **Module Re-export**: `code/aiosh-rust/aiosh-core/src/lib.rs`
   - Declared `pub mod fs_layout_service;`.
   - Re-exported `FilesystemLayoutService`, `FilesystemLayoutStore`, `LayoutDiff`, `MountDiffItem`, `PartitionDiffItem`, `TargetEvaluation`.
3. **Test Stub**: `code/aiosh-rust/aiosh-core/tests/test_fs_layout_service.rs`
   - Exercises initialization of `FilesystemLayoutService` and `FilesystemLayoutStore`.
   - Confirms default active layout ID `"aios-uefi-standard-v1"` and empty constructor.

---

## 2. Compilation & Test Verification

```
$ cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_fs_layout_service
   Compiling aiosh-core v0.1.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 29.53s
     Running tests\test_fs_layout_service.rs

running 1 test
test test_fs_layout_service_scaffold_initialization ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Acceptance criteria satisfied:
- Project builds with zero errors.
- New interfaces exist and are referenced by test stub.
