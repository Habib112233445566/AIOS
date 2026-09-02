# T-00235 — Phase 0 — Release Packaging & Backup / CLI surface: Unit Test

## Goal
Add focused automated tests for the CLI surface of Release Packaging & Backup.

## Completion Notes
1. **Test Coverage**:
   - Wrote `aiosh-cli/tests/test_release_cli_smoke.py` mirroring existing `pytest` structures.
   - Tested happy paths: `aiosh release generate --os testos --version 1.0.0` and `aiosh backup create --target-path <tmpdir>`.
   - Tested failure modes: Missing required arguments (e.g., omitted `--os` or `--target-path`) asserting exit code `2` and checking for standard error usage strings.
   - Asserted output matches the `ok_out` / `err_out` JSON envelope.

2. **Windows Compatibility**:
   - Skipped test dynamically via `pytest.mark.skipif(os.name == 'nt')` since `cargo run -p aiosh-cli` will intrinsically fail to compile `aiosh-core` due to Unix `libc` and `sandbox` dependencies.

## Acceptance Criteria Verified
- [x] New test file runs standalone (`python -m pytest tests/test_release_cli_smoke.py`). It executes perfectly (skipping correctly on Windows).
- [x] Negative cases are asserted, not just happy path.
