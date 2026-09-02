# T-00645 — Repository Health / MCP/API surface: Unit Test

## 1. Unit Test Scope
This task verifies JSON-RPC tool declaration and execution for `aios.repo.health` in `code/aiosh-mcp/tests/test_repo_mcp_smoke.py`.

## 2. Test Execution & Coverage
1. **`test_mcp_tools_list_repo_health`**:
   - Asserts `aios.repo.health` is advertised via `tools/list` with its input schema properties.
2. **`test_mcp_repo_health_default_path`**:
   - Asserts `tools/call` for `aios.repo.health` executes without errors and returns `report` containing checks.
3. **`test_mcp_repo_health_custom_temp_repo`**:
   - Asserts `tools/call` for `aios.repo.health` on a temp directory returns evaluated checks.

## 3. Test Verification Output
```text
PASS: aios.repo.health present and valid in tools/list
PASS: aios.repo.health tool call on repository root
PASS: aios.repo.health tool call on temp directory

ALL MCP REPO HEALTH SMOKE TESTS PASSED!
```
