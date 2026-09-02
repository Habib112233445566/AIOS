# T-00306 — Release Packaging & Backup: Recovery & Validation Integration

## Overview
We integrated the recovery and validation functionality into the `aiosh-mcp` agent interface, making it available as tools for AI actors interacting with the system.

## MCP Tools Added

1. **`aios.release.validate`**
   - **Endpoint**: Takes `artifact_path` and `expected_hash`.
   - **Behavior**: Verifies the ISO exists and matches expected criteria, returning a structured JSON response.

2. **`aios.backup.validate`**
   - **Endpoint**: Takes `backup_path`.
   - **Behavior**: Performs structural integrity verification on the target ZIP archive.

3. **`aios.backup.restore`**
   - **Endpoint**: Takes `backup_path`, `target_dir`, and `grant_id`.
   - **Behavior**: Securely extracts the backup ZIP to the `target_dir`, enforces PEP authorization internally, and emits exactly one `AuditRowInput` upon successful restoration.

## Validation
- Recompiled `aiosh-mcp` and verified successful parsing and type-checking of the tool definitions.
- Verified that all tool parameters correctly map into the backend Rust `ReleaseCtx` structure.
- The task is complete.
