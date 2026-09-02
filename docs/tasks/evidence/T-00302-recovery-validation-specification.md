# T-00302 — Release Packaging & Backup: Recovery & Validation Specification

## Core Operations

### 1. `validate_release`
- **Goal**: Verify that a generated `.iso` is structurally sound.
- **Inputs**: `artifact_path: &str`, `expected_hash: &str`
- **Behavior**: Because we want to minimize external binary dependencies (e.g., `isoinfo`), we will read the physical file at `artifact_path`, compute its SHA256 hash in memory, and verify it exactly matches the `expected_hash` (the hash emitted during generation).
- **Outputs**: `Result<(), String>`. Returns `Ok` if hashes match, `Err` if the file is missing or hashes diverge.

### 2. `validate_backup`
- **Goal**: Verify that a generated `.zip` backup is not corrupted.
- **Inputs**: `backup_path: &str`
- **Behavior**: Uses the `zip::ZipArchive` API to open the file and iterate over its central directory. If `zip::ZipArchive::new` succeeds and we can iterate over the names without an `InvalidArchive` error, it is deemed structurally sound.
- **Outputs**: `Result<(), String>`. Returns `Ok` if valid, `Err` if corrupted.

### 3. `restore_backup`
- **Goal**: Extract a `.zip` backup into a target directory (Recovery).
- **Inputs**: `ctx: &mut ReleaseCtx`, `backup_path: &str`, `target_dir: &str`
- **Behavior**: 
  - Validates `target_dir` is empty or creates it if it doesn't exist. Refuses to extract over an existing non-empty directory to prevent merged state corruption.
  - Opens `backup_path` via `zip::ZipArchive` and extracts files into `target_dir`.
  - **Security**: Must actively prevent zip-slip (path traversal) by stripping or rejecting zip entries containing `..` or absolute paths (`/`).
  - Emits exactly one `AuditRow` to the system ledger via `aios.backup.restore`.
- **Outputs**: `Result<(), String>`.

## Audit & PEP Effects
- `restore_backup` is a state-changing action that overwrites filesystem state. It MUST be protected by the `pep::check_release_policy` for `aios.backup.restore` (meaning an active cryptographic grant token is required).
- Validation actions (`validate_release`, `validate_backup`) are read-only and do not mutate state, thus they do not require an audit row or a PEP grant, but may be explicitly requested by agents.
