# T-02435: Sandbox Enforcement MCP/API Surface Unit Test

## 1. Unit Test Scope
This document records verification evidence for task T-02435: Sandbox Enforcement MCP/API Surface Unit Test.
Focused automated unit tests were created in `code/aiosh-mcp/tests/test_sandbox_mcp.py` to validate JSON-RPC 2.0 requests over stdio.

## 2. Test Cases Asserted
1. `test_mcp_sandbox_tool_registration`: Asserts discovery of `aios.sandbox.profiles`, `aios.sandbox.probe`, and `aios.sandbox.exec` in `tools/list`.
2. `test_mcp_sandbox_profiles`: Asserts presence of default factory profiles (`standard`, `strict`, `permissive`).
3. `test_mcp_sandbox_probe`: Asserts detection of kernel sandbox capabilities and platform string.
4. `test_mcp_sandbox_exec_success`: Asserts end-to-end execution of a process with exit code 0 and captured standard output.
5. `test_mcp_sandbox_exec_empty_command`: Negative case asserting rejection of empty or blank commands (`ERR_SANDBOX_EMPTY_COMMAND`).
6. `test_mcp_sandbox_exec_traversal_cwd`: Negative case asserting rejection of working directories containing `..` (`ERR_SANDBOX_INVALID_PATH`).
7. `test_mcp_sandbox_exec_unknown_profile`: Negative case asserting rejection of unregistered profiles (`ERR_SANDBOX_PROFILE_NOT_FOUND`).

## 3. Results
```text
============================== 7 passed in 0.56s ==============================
```
All unit tests pass in isolation.
