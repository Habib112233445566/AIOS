# T-00215 — Phase 0 — Release Packaging & Backup / Data Model: Unit Test

## Goal
Add focused automated tests for the data model of Release Packaging & Backup, covering valid input, invalid input, boundary values, and primary failure modes.

## Completion Notes
1. **Rust Tests (`aiosh-core/src/release.rs`)**:
   - `test_generate_release_valid`: Verifies correct canonical hash generation, `.iso` path generation, and assert the `AuditRing` was successfully mutated by checking the new head hash.
   - `test_generate_release_boundary`: Checks boundary condition where `manifest` fields are empty strings or empty arrays, verifying it still calculates a stable hash and generates a deterministic path.
   - `test_create_backup_valid`: Asserts correct `.zip` timestamped output format for snapshots.

2. **Python MCP Parity Tests (`aiosh_mcp/tests/test_release_smoke.py`)**:
   - Upgraded `test_generate_release_stub` to `test_generate_release_valid` making use of the SQLite test db.
   - Included negative boundary checks.
   - Asserted that `audit_client.tail(conn, 1)` yields the correct `aios.release.generate` and `aios.backup.create` logs with proper tool inputs and args matched.
   - Fixed `PermissionError` file-lock issue during `pytest` teardown natively.

## Acceptance Criteria Verified
- [x] New test file runs standalone and passes (`python -m pytest` exits with code 0).
- [x] Negative cases are asserted, not just happy path.
- [x] Asserts observable behavior (DB state and hashes), not just mock implementations.
