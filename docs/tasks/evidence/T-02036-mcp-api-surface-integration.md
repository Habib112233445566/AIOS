# T-02036: Capability Model / MCP/API Surface — Integration

**Task ID**: `T-02036`  
**Phase**: Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic**: Sub-Epic 4: Capability Model / MCP/API Surface  
**Status**: COMPLETED  
**Date**: 2026-09-20  

---

## 1. Integration Summary

The Capability Model MCP/API surface has been integrated end-to-end into the AIOS production environment:
1. **MCP Server Integration**:
   - Registered 7 tools in `Server::tool_manifest(&self)` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
   - Wired tool dispatch through `dispatch::recorded_call` ensuring PEP evaluation and Audit Ring hash chaining.
2. **Cross-Surface Parity**:
   - Standard JSON Schema Draft-07 schemas exposed over JSON-RPC `tools/list` and invoked via `tools/call`.
   - Backing store uses canonical JSON with atomic file persistence (`CapabilityService::save_to_path`).
3. **Integration Smoke Test**:
   - Implemented `code/aiosh-mcp/tests/test_capability_mcp_smoke.py`.
   - Executed against compiled `aiosh-mcp.exe` binary.
   - Verified tool discovery, empty list query, unauthorized issuance rejection, root issuance, ID lookup, privilege escalation rejection on attenuation, valid attenuation, quota consumption on check, unauthorized right denial, cascade revocation, post-revocation denial, and leaf pruning.

---

## 2. Test Execution Output

```text
Running Capability MCP Smoke against binary: C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\target\debug\aiosh-mcp.exe
TEST: tool registration via tools/list ... OK
TEST: capability full lifecycle over MCP JSON-RPC ... OK
ALL TESTS PASSED
```
