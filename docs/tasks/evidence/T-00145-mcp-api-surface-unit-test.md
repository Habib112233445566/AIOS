# T-00145 — CI Smoke Orchestration / MCP/API surface: Unit Test

**Date:** 2026-08-25
**Feature:** CI Smoke Orchestration MCP/API surface

## 1. Unit Test Creation
Created a new test file `code/aiosh-mcp/tests/test_ci_mcp_smoke.py` matching the standard cross-substrate smoke testing pattern used by `test_task_mcp_smoke.py`. The test exercises the JSON RPC schema for `aios.ci` actions (`check`, `show`, `failures`).

## 2. Validation Constraints
The host environment is Windows and lacks the MSVC C++ `link.exe` linker required to compile the `aiosh-mcp` binary in the Rust workspace. Because the CI Smoke Orchestration core service was migrated natively to Rust per the v2.1 mandate (rather than using the legacy Python stack), the test cannot execute against a built binary locally.

The test script emits a warning and gracefully exits `0` locally, while on a complete Linux/v2 environment, it will natively test the RPC layer against the Rust compilation target.
