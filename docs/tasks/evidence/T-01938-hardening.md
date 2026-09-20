# Task Evidence: T-01938 - System Update / MCP/API surface: Hardening

- **Task**: `T-01938`
- **Sub-Epic**: `Sub-Epic 4: Model Context Protocol (MCP) & API Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Hardening
Implemented defensive hardening across the System Update MCP tool dispatch surface in `code/aiosh-rust/aiosh-mcp/src/main.rs`:

1. **Parent Directory Component (`..`) Traversal Rejection**:
   - `resolve_update_service`: rejects any `state_dir` or `staging_dir` containing `..` parent directory traversal components.
   - `aios.update.check`: rejects `manifest_path` containing `..`.

2. **Symlink Metadata Defense**:
   - `aios.update.check`: inspects `std::fs::symlink_metadata(manifest_path)` and immediately rejects any manifest file that is a symlink, preventing symlink spoofing and TOCTOU races.

3. **Manifest Size Bound**:
   - `aios.update.check`: enforces a strict 1MB size limit (`meta.len() <= 1_048_576`) on manifest files prior to reading bytes into memory.

4. **Version String Sanitization & Bound**:
   - `aios.update.confirm`: enforces that version strings do not exceed 64 characters and contain zero control characters and zero whitespace characters.

5. **Resource Cleanup & Atomic Persistence**:
   - All state updates persist through atomic temporary files with fsync and atomic rename.
   - Errors always produce explicit, structured JSON error envelopes with honest audit rows emitted via `dispatch::recorded_call`.

## Verification
- Unit test suite: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp test_system_update_mcp_tools`.
- Integration smoke suite: `python code/aiosh-mcp/tests/test_system_update_mcp_smoke.py`.
