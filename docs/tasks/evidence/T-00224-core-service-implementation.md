# T-00224 — Phase 0 — Release Packaging & Backup / Core Service: Implementation

## Goal
Implement the minimal working behavior for the core service of Release Packaging & Backup.

## Completion Notes
1. **Python Implementation (`aiosh-mcp/aiosh_mcp/release.py`)**:
   - `physical_generate_iso`: Implemented file I/O to create a mock ISO file. Subprocess invocation is guarded by a `try/except` block, ensuring fail-open logic doesn't crash silently, and errors bubble up to the Dispatch gate.
   - `physical_create_zip`: Implemented a recursive walk of the source directory using `zipfile`. Applied the `include_audit` and `include_memory` filters (by mutating `dirs.remove()`), writing exactly the permitted files to `ZIP_DEFLATED`.
   - Verified that all imports resolve perfectly (e.g. `import zipfile`, `import subprocess`).

2. **Rust Implementation (`aiosh-core/src/release.rs`)**:
   - `physical_generate_iso`: Mocked with `std::fs::File` and `write_all`. The directory is created automatically via `create_dir_all`.
   - `physical_create_zip`: Leveraged the `zip` crate to open a `ZipWriter` over a `File`. For the initial scaffold, it writes a `aios_backup_manifest.json` dummy file since the recursive directory traversal is largely handled in python for now (the primary target for the tool MCP wrapper).

## Acceptance Criteria Verified
- [x] Targeted test passes (verified `pytest tests/test_release_smoke.py` in `aiosh-mcp`).
- [x] Kept all audit/PEP invariants (dispatch block hasn't been modified). No regression.
