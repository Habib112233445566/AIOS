# T-00361 — Dependency & Toolchain Pinning / automated tests: Research

## Goal
Determine the strategy for adding automated end-to-end smoke tests for the `Dependency & Toolchain Pinning` epic, ensuring the CLI and MCP surfaces are verified within the overarching `ci/run_all_smokes.sh` orchestrator.

## Facts & Existing Patterns
1. **Orchestrator**: `ci/run_all_smokes.sh` wraps `tools/ci_run.py`, which iterates sequentially through `tools/ci_suites.py`.
2. **Current State**: The `tools/ci_suites.py` registry contains suites for task ledger, pentest, retention, sandbox, classifier, release packaging, and metrics. However, there are no suites executing the new toolchain functionality.
3. **CLI Pattern**: CLI smoke tests reside in `code/aiosh-cli/tests/` (e.g., `test_task_cli_smoke.py`). They typically use Python's `subprocess` to invoke the `target/debug/aiosh` binary and assert on stdout/stderr JSON responses.
4. **MCP Pattern**: MCP smoke tests reside in `code/aiosh-mcp/tests/` (e.g., `test_task_mcp_smoke.py`). They construct JSON-RPC requests, spawn the `aiosh-mcp` binary, pass the requests via `stdin`, and parse the JSON-RPC responses.

## Action Plan
1. **Scaffold (T-00363)**: Create two new files:
   - `code/aiosh-cli/tests/test_toolchain_cli_smoke.py`
   - `code/aiosh-mcp/tests/test_toolchain_mcp_smoke.py`
2. **Implementation (T-00364)**:
   - Implement `test_toolchain_cli_smoke.py` to assert `aiosh toolchain check` parses the root `config/toolchain.json` correctly.
   - Implement `test_toolchain_mcp_smoke.py` to assert the `aios.toolchain.check` JSON-RPC method returns `ok: true`.
3. **Integration (T-00366)**:
   - Add the new scripts to `tools/ci_suites.py` as `toolchain_cli_smoke` and `toolchain_mcp_smoke`.
4. **Verification (T-00370)**:
   - Run `ci/run_all_smokes.sh` and ensure all tests pass (including the two new suites).
