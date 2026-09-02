# T-00745 — Secrets & Access Hygiene / MCP/API surface: Unit Test

## 1. Test Deliverables
- Added unit tests in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
  - `aiosh_mcp::tests::test_mcp_secrets_tools_execution`: Tests tool schema registration and execution for `aios.secrets.scan` and `aios.secrets.check`.
  - Asserted tool presence in `tool_manifest()`.
- Extended `tools/test_secrets_suites.py` with criteria `K6` (`test_k6_mcp_surface`).
- Verified execution via standalone test suite.

## 2. Test Execution Output
```text
[+] K1 data model integrity
[+] K2 private key scanner
[+] K3 API token scanner
[+] K4 config & env credentials scanner
[+] K5 CLI surface commands & options
[+] K6 MCP tool schemas & execution

PASS: secrets_suites criteria (K1..K6)
```
