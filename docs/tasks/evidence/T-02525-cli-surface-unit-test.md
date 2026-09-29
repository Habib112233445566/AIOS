# Evidence: T-02525 Privilege Escalation Prevention CLI Surface Unit Tests

- **Task**: `T-02525`
- **Sub-Epic**: Privilege Escalation Prevention / CLI surface
- **Date**: 2026-09-29
- **Status**: PASSED

## Summary of Unit Tests
1. Created unit tests in `mod privilege_cli_tests` in `code/aiosh-rust/aiosh-cli/src/main.rs`:
   - `test_privilege_cli_help_and_unknown`: Verifies help flag output (code 0) and unknown subcommands (code 2).
   - `test_privilege_cli_path_hygiene`: Verifies rejection of relative path traversal (`..`), control characters, and paths exceeding 1024 bytes (code 2).
   - `test_privilege_cli_lifecycle`: Verifies complete lifecycle operations:
     - `status`: Creates initial User context (code 0).
     - `elevate`: Denies unauthorized upward transitions (code 1), blocks `SystemKernel` (code 1), allows grant-backed escalation (code 0).
     - `check`: Validates holding or lacking specific capabilities (codes 0 and 1).
     - `drop`: Safely demotes tier and prunes capabilities (code 0).
     - `revoke`: Restores registered base level (code 0).
     - `list`: Lists registered actors (code 0).
2. Clean compilation across workspace (0 warnings, 0 errors).
