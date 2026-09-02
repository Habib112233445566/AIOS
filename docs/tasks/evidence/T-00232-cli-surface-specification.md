# T-00232 — Phase 0 — Release Packaging & Backup / CLI surface: Specification

## Goal
Specify the exact contract for the CLI surface of Release Packaging & Backup.

## Inputs & Outputs

### 1. `aiosh backup create`
**Command Signature:**
`aiosh backup create --target-path <path> [--include-audit <true|false>] [--include-memory <true|false>]`

**Inputs:**
- `--target-path` (required): The path on the filesystem to capture.
- `--include-audit` (optional, default `true`): Whether to include the `audit` directory in the zip archive.
- `--include-memory` (optional, default `false`): Whether to include memory/RAM dumps in the zip archive.

**Outputs (JSON):**
```json
{
  "ok": true,
  "subcommand": "backup create",
  "data": {
    "backup_path": "aios_backup_2026-08-26T20-30-00Z.zip"
  }
}
```

### 2. `aiosh release generate`
**Command Signature:**
`aiosh release generate --os <target_os> --version <version> [--components <c1,c2...>]`

**Inputs:**
- `--os` (required): The target OS architecture (e.g. `ubuntu`, `windows`).
- `--version` (required): The version string for the release (e.g. `1.0.0`).
- `--components` (optional, default `core`): A comma-separated list of components to include in the release manifest.

**Outputs (JSON):**
```json
{
  "ok": true,
  "subcommand": "release generate",
  "data": {
    "artifact_path": "output/release/aios_ubuntu_1.0.0.iso",
    "hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  }
}
```

## Error Cases
If any backend core function (`create_backup` or `generate_release`) returns an error (e.g., directory permission denied, bad target), the CLI process will:
1. Return exit code `1`.
2. Output a standard error envelope to `stderr`:
```json
{
  "ok": false,
  "subcommand": "backup create",
  "error": "Physical backup failed: Permission denied"
}
```
3. Emit an immutable `error` row to the AuditRing reflecting the failed operation.

## Persistence Effects
- The CLI command instantiates the same `ReleaseCtx` used by the MCP service, which triggers `aiosh_core::audit::AuditRing::write(...)`. 
- No additional `emit()` is needed within `main.rs`, as the core layer satisfies ADR-0035 internally. 

## Acceptance Criteria Verified
- [x] Spec covers happy path, failure path, and audit effects.
- [x] Spec is reviewable without reading the implementation.
