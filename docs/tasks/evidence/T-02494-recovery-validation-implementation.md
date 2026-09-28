# Task T-02494 Evidence: Sandbox Recovery & Validation Implementation

## Goal
Implement the minimal working behavior for Sandbox Enforcement Recovery & Validation (`SandboxRecoveryManager`).

## Implementation Summary
1. **Validation Engine (`code/aiosh-rust/aiosh-core/src/sandbox_recovery.rs`)**:
   - `validate()`: Verifies presence and structural integrity of the 3 immutable factory profiles (`standard`, `strict`, `permissive`), checks in-memory registered profile invariants, and optionally scans custom profile JSON manifests in a custom directory.
   - `validate_profile_file()`: Verifies file size bounds (<= 64 KiB), forbids directory traversal (`..`), checks JSON syntax, and validates resource limit boundaries.
2. **Recovery Strategies**:
   - `DryRun`: Reports diagnostic validation issues without mutating profiles or disk.
   - `RestoreFactoryDefaults`: Re-registers pristine baseline factory profiles via `upsert_profile()`.
   - `QuarantineAndReset`: Moves corrupted custom manifests into `.quarantine_<timestamp>` directory and restores clean factory profiles.
3. **Service Integration**:
   - Added `upsert_profile()`, `validate_state()`, and `recover_state()` to `SandboxService`.
   - Added `SandboxService::empty()` constructor for testing damaged/uninitialized configurations.
4. **Verification**:
   - Verified 4/4 tests pass in `test_sandbox_recovery.rs`.
   - 0 compiler warnings across workspace.
