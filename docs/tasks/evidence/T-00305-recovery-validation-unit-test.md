# T-00305 — Release Packaging & Backup: Recovery & Validation Unit Test

## Overview
We added unit tests for the recovery and validation functionality in `aiosh-core/src/release.rs`. These tests ensure that the validation logic behaves correctly for various input types, boundaries, and failure modes, specifically focusing on enforcing PEP gates and protecting the system state.

## Test Coverage

### `test_validate_release_invalid_hash`
- Tests `validate_release` with a malformed expected hash (e.g., short string).
- Asserts that it correctly rejects the validation early without attempting to open any files.

### `test_restore_backup_refuses_non_empty_dir`
- Simulates an attempt to restore a backup into a target directory that already contains files.
- Asserts that the function halts execution early and returns an error containing `"Target directory is not empty"` to prevent overwriting existing data.
- Bypasses the PEP gate by explicitly passing a mock grant (`Some("grant_token")`) to ensure the directory logic is reachable.

### `test_restore_backup_requires_grant_if_checked`
- Asserts the PEP gate security invariant.
- Attempts to call `restore_backup` with `None` as the grant token.
- Asserts that the execution is blocked with the error `"requires explicit PEP grant"`, demonstrating that the tool's classification (`aios.backup.restore`) is correctly integrated into `pep::is_irreversible`.

## Validation
- Ran `cargo test -p aiosh-core`.
- All 80 core tests passed. The 3 new recovery validation tests passed correctly and reliably.
- No warnings or errors were introduced into the code base.
- The task is complete.
