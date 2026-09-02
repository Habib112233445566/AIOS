# T-00223 — Phase 0 — Release Packaging & Backup / Core Service: Scaffold

## Goal
Create the module skeleton and interfaces for the core service of Release Packaging & Backup.

## Completion Notes
1. **Rust Scaffold (`aiosh-core/src/release.rs`)**:
   - Added `physical_generate_iso(manifest: &PackageManifest, artifact_path: &str) -> Result<(), String>`.
   - Added `physical_create_zip(snapshot: &BackupSnapshot, backup_path: &str) -> Result<(), String>`.
   - Both functions are stubbed with `unimplemented!()` macro to fail loudly during this scaffold phase.
   - Wired up the `zip = "2"` dependency in `Cargo.toml`.
   - Fixed a pre-existing private export compile error where `CFlags` was incorrectly imported from `crate::audit` instead of `crate::types`.
2. **Python Scaffold (`aiosh-mcp/aiosh_mcp/release.py`)**:
   - Added `physical_generate_iso(manifest: PackageManifest, artifact_path: str) -> None`.
   - Added `physical_create_zip(snapshot: BackupSnapshot, backup_path: str) -> None`.
   - Both functions throw `NotImplementedError` currently.

## Acceptance Criteria Verified
- [x] Project builds/imports with zero errors (modulo pre-existing Windows Linux syscall failures noted previously). The `aiosh_mcp.release` import test passes natively.
- [x] New interfaces exist and are structured according to the specification.
