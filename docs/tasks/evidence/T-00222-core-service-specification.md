# T-00222 — Phase 0 — Release Packaging & Backup / Core Service: Specification

## Goal
Specify the exact contract for the core service of Release Packaging & Backup (file I/O phase).

## Interfaces Reused vs New
- **Reused**: 
  - `aiosh_mcp.release.generate_release` and `aiosh_mcp.release.create_backup` signatures remain identical.
  - Audit Ring integration (`dispatch_mod.commit`, `audit_client.write_audit_row`) remains the sole mutator for persistence.
- **New**:
  - `zip` crate dependency for robust archiving in Rust.
  - `genisoimage` CLI dependency for assembling releases.
  - `zipfile` (Python stdlib) and `subprocess` for MCP parity file operations.

## Contract Specification

### 1. Release Generation (ISO)
- **Input**: `target_os` (string), `components` (list of strings), `version` (string).
- **Operation**:
  1. Validates inputs implicitly via strict character filtering (no path traversal elements allowed).
  2. Synthesizes an output filename `output/release/aios_{target_os}_{version}.iso`.
  3. Executes `genisoimage -o {filename} -V "AIOS Release" -R -J {source_path}`.
- **Output**: 
  - Success: Returns the canonical SHA256 of the ISO and the artifact path.
  - Failure: If `genisoimage` is missing or fails (returns non-zero exit code), raises a descriptive error (e.g., `genisoimage failed with status 1: ...`).
- **Audit Effects**: One row emitted, recording `outcome="success"` (and the SHA256) or `outcome="error"` (with stderr details).
- **Timeouts/Caps**: External process is forcefully killed if it exceeds 300 seconds (5 minutes).

### 2. System Backup (ZIP)
- **Input**: `target_path` (string, the root to backup), `include_audit` (bool), `include_memory` (bool).
- **Operation**:
  1. Synthesizes a zip path `aios_backup_{timestamp}.zip`.
  2. Recursively walks the `target_path` (while skipping `/audit/` or `/memory/` if booleans are false).
  3. Streams file data directly into the zip archive via the `zip` crate (Rust) or `zipfile` (Python).
- **Output**:
  - Success: Returns the artifact path.
  - Failure: If a file cannot be read due to permissions or missing directory, raises a clear IO error.
- **Audit Effects**: One row emitted on success/fail containing the exact exception string on failure.
- **Timeouts/Caps**: Walk recursion capped at 100 levels deep to prevent cyclic symlinks or infinite nesting.

## Edge Cases and Fallbacks
- If `genisoimage` is missing, the service fails gracefully with an explicit error indicating the missing dependency, logging a clean `error` row.
- In both subsystems, failures to write the actual file to disk (e.g. disk full) throw exceptions that are cleanly trapped by the PEP gate dispatch wrapper, satisfying fail-open observability.
