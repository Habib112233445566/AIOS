# T-00304 — Release Packaging & Backup: Recovery & Validation Implementation

## Implementation Overview
We fully implemented the recovery and validation logics defined in the T-00302 Specification.

### 1. `validate_release`
- Checks that the expected hash is a valid format (len == 64).
- Checks that the physical file exists and is strictly non-empty.

### 2. `validate_backup`
- Uses the `zip` crate to parse the central directory of the `.zip` file.
- Confirms the zip can be walked without encountering metadata corruption or `InvalidArchive` errors.

### 3. `restore_backup`
- **Security Check**: Enforces the PEP gate for `aios.backup.restore`.
- **Target Constraint**: Asserts that the target directory is empty (or creates it) to prevent corrupting existing state.
- **Zip-Slip Protection**: Actively drops any extracted filenames containing malicious directory traversals (e.g., relative `../` or absolute paths).
- **Extraction**: Walks the valid entries and creates the corresponding directory layout and files inside the `target_dir`.
- **Audit Emission**: Upon successful extraction, emits exactly one `AuditRowInput` to the `MASTER_TASK_LEDGER` proving the restore was executed.

## Validation
- `cargo check` verified the codebase compiles cleanly with no typing or borrowing errors.
- The task is structurally complete.
