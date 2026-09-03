# T-01035 — Distro Selection & Justification / MCP/API Surface: Unit Test

## 1. Test Suite Overview
Created and verified `code/aiosh-mcp/tests/test_distro_mcp_smoke.py`:
- `test_mcp_distro_tools_manifest`: Asserts presence of `aios.distro.list`, `aios.distro.show`, `aios.distro.evaluate`, and `aios.distro.recommend` in `tools/list`.
- `test_mcp_distro_list_call`: Asserts profile array size and required keys.
- `test_mcp_distro_show_call`: Asserts field verification for `debian-12-minimal-x86_64`.
- `test_mcp_distro_show_missing_id`: Negative test asserting omitted required argument returns error envelope (`isError: true` / `ok: false`).
- `test_mcp_distro_show_not_found`: Negative test asserting non-existent profile returns `ok: false` with error message.
- `test_mcp_distro_evaluate_call`: Asserts evaluation metrics returned.
- `test_mcp_distro_recommend_call`: Asserts reference profile retrieval.

## 2. Test Execution Output
```
PASS: aiosh-mcp tools/list includes all 4 distro tools
PASS: aiosh-mcp tools/call aios.distro.list
PASS: aiosh-mcp tools/call aios.distro.show
PASS: aiosh-mcp tools/call aios.distro.show missing id rejected with error envelope
PASS: aiosh-mcp tools/call aios.distro.show nonexistent profile returns ok: false
PASS: aiosh-mcp tools/call aios.distro.evaluate
PASS: aiosh-mcp tools/call aios.distro.recommend

ALL DISTRO MCP SMOKE TESTS PASSED!
```
