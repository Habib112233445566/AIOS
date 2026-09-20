# T-01836: Network Bootstrap / MCP/API Surface: Integration

## 1. Overview
- **Task ID**: `T-01836`
- **Sub-Epic**: 4 (MCP/API Surface)
- **Goal**: Integrate the MCP/API surface of Network Bootstrap with the surrounding system.

---

## 2. Integration Details
1. **Tool Registration in `aiosh-mcp`**:
   - All 7 network tools (`aios.network.list`, `show`, `routes`, `dns`, `state`, `up`, `down`) are registered and advertised via `tools/list`.
   - Tool schemas conform to standard JSON-RPC 2.0 / MCP specifications.
2. **Cross-Surface Parity Verification**:
   - Implemented `code/aiosh-mcp/tests/test_network_mcp_smoke.py`.
   - Validated end-to-end JSON-RPC invocation of each tool against live compiled binary `aiosh-mcp`.
   - Compared CLI output (`aiosh net list --json`) with MCP output (`aios.network.list`) on identical mock filesystems, asserting 100% data parity.
3. **Security Invariants Verified**:
   - Path hygiene: Refusal of paths exceeding 1024 chars or containing control characters.
   - Interface name validation: Refusal of invalid or malicious interface strings.
   - Link mutations (`up`, `down`) update state and emit consequential audit rows.

## 3. Verification Results
```text
Starting Network Bootstrap MCP Integration Smoke Suite...
PASS: test_tool_registration (all 7 network tools advertised)
PASS: test_path_hygiene_and_validation
PASS: test_mock_lifecycle_and_cross_surface_parity
ALL 7 NETWORK BOOTSTRAP MCP INTEGRATION TESTS PASSED.
```
