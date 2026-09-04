# T-01193: Base Image Build Recovery & Validation Scaffold

**Date:** 2026-09-04  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Base Image Build / Recovery & Validation  
**Task ID:** T-01193  

## 1. Scaffold Deliverables
- Created module `code/aiosh-rust/aiosh-core/src/base_image_recovery.rs`.
- Defined core data models:
  - `RecoveryAction`: `LoadedExisting`, `CreatedDefaultFresh`, `RecoveredFromBackup`.
  - `BaseImageValidationReport`: Tracks `healthy`, `total_manifests`, `valid_manifests`, `invalid_manifests`, `errors`, `warnings`, and enforces invariants `RV1..RV3`.
- Defined interface stubs:
  - `validate_manifest`
  - `validate_store`
  - `load_or_recover`
  - `repair_store`
- Registered `pub mod base_image_recovery;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Verified clean build via `cargo check` across the full workspace.
