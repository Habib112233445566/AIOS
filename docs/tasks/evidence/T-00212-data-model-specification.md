# T-00212 — Phase 0 — Release Packaging & Backup / Data Model: Specification

## Goal
Specify the exact contract for the data model of Release Packaging & Backup.

## 1. Release Packaging Data Model (AIOS-Specific)

### Interfaces
- **New Interface**: `PackageManifest`
  - Defines the required components for a reproducible AIOS release (ISO build or `aiosh install` script).
- **Reused Interfaces**: `ProjectManifest` (from `PROJECT_MANIFEST.yaml`).

### Contract
- **Inputs**:
  - `target_os`: String enum (`debian-13`, `ubuntu-24.04-lts`)
  - `components`: Array of required subsystems (e.g., `aiosh-mcp`, `aiosh-rust`, `kde-plasma-6`, `wine-11.x`)
  - `version`: Semantic version string.
- **Outputs**:
  - A deterministic build artifact (e.g., ISO, signed tarball, or Debian repository manifest).
  - A SHA-256 checksum for the generated artifact.
- **Persistence Effects**:
  - The build system writes the final artifact to an `output/release/` directory.
- **Error Cases**:
  - `MissingDependencyError`: A declared component fails to resolve in the host package manager.
  - `ChecksumMismatchError`: The generated artifact does not match expected deterministic hashes.

## 2. Backup Data Model (AIOS-Specific)

### Interfaces
- **Reused Interfaces**: `workspace.py sync` pipeline (from T-00008), Audit Ring schema (from Sprint 3).
- **New Interface**: `BackupSnapshot`

### Contract
- **Inputs**:
  - `target_path`: The root directory of the workspace or system state (e.g., `/var/aios/state` or workspace root).
  - `include_audit`: Boolean (whether to bundle `audit-archive/segment-*.jsonl`).
  - `include_memory`: Boolean (whether to backup the encrypted semantic memory SQLite database).
- **Outputs**:
  - A compressed snapshot file named `aios_backup_<timestamp>.zip`.
  - A JSON metadata manifest containing the snapshot details and hashes.
- **Persistence Effects**:
  - Reads from the filesystem.
  - Archives contents while preserving SQLite transactional consistency.
  - Syncs the archive to the remote storage (e.g. Cloudflare R2) using `workspace.py sync`.
- **Error Cases**:
  - `TransactionLockError`: Unable to safely backup the SQLite database due to concurrent writes.
  - `SyncFailedError`: The upload to external storage fails (network or auth error).
