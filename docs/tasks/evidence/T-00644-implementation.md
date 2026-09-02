# T-00644 — Repository Health / MCP/API surface: Implementation

## 1. Implementation Scope
This task implements the `aios.repo.health` MCP tool handler in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. Implementation Deliverables
- Implemented `aios.repo.health` in `call_tool()`:
  - Extracts optional `repo_path` (defaulting to current directory `.`).
  - Calls `aiosh_core::repo_health_service::check_repo_health`.
  - Envelopes response into `{"ok": true, "tool": "aios.repo.health", "report": report}`.
  - Emits audit record via `dispatch::recorded_call`.
- Added unit test `test_mcp_repo_health_execution` asserting tool manifest inclusion and successful tool execution.

## 3. Test Verification Output
```text
running 1 test
test tests::test_mcp_repo_health_execution ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.52s
```
