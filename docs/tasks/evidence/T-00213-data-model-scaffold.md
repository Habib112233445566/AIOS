# T-00213 — Phase 0 — Release Packaging & Backup / Data Model: Scaffold

## Goal
Create the module skeleton and interfaces for the data model of Release Packaging & Backup.

## Completion Notes
1. **Module Skeleton Added:**
   - Created `code/aiosh-rust/aiosh-core/src/release.rs` with `PackageManifest` and `BackupSnapshot` types.
   - Added corresponding Python implementations in `code/aiosh-mcp/aiosh_mcp/release.py` to maintain cross-substrate interface definitions.
2. **Export Configuration:**
   - Exposed Rust module via `pub mod release;` in `aiosh-core/src/lib.rs`.
   - Exposed Python module via `__init__.py` in `aiosh_mcp/`.
3. **Function Stubs & Type Safety:**
   - Typed interfaces `generate_release` and `create_backup` were defined but remain unimplemented (`unimplemented!()` in Rust, `NotImplementedError` in Python).
4. **Validation (Tests):**
   - Wrote explicit stub-failure tests in Rust (`#[test] #[should_panic]`) to verify the interface and failure modes.
   - Wrote Pytest checks (`tests/test_release_smoke.py`) ensuring `NotImplementedError` is properly raised.
   - Project successfully compiles (`cargo test`) and Python module imports gracefully (`python -m pytest tests/test_release_smoke.py`).

## Acceptance Criteria Verified
- [x] Project builds/imports with zero errors.
- [x] New interfaces exist and are referenced by at least one call site or test stub.
