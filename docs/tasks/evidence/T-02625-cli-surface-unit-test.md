# T-02625: Secrets Handling CLI Surface Unit Test

- **Task**: `T-02625`
- **Sub-Epic**: Secrets Handling / CLI surface
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Unit Test Suite Summary
- Implemented `secret_cli_tests` module in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  1. `test_secret_cli_help_and_unknown`: Validated help invocations (`--help`, `-h`, empty args) returning code 0, and unrecognized commands returning code 2.
  2. `test_secret_cli_path_hygiene`: Validated path traversal detection (`..`) returning exit code 2.
  3. `test_secret_cli_lifecycle`: Validated full CLI command lifecycle:
     - Missing flag syntax error (code 2).
     - Store secret success (code 0).
     - Get secret with masked value by default, `--expose` for plaintext, and `--json` format (code 0).
     - Get non-existent secret returning error (code 1).
     - List secrets by kind and scope without payload leakage (code 0).
     - Rotate secret payload (code 0).
     - Revoke secret (code 0) and assert subsequent retrieval denied (code 1).
- Test execution output:
  - 3 passed; 0 failed; finished in 0.28s.
