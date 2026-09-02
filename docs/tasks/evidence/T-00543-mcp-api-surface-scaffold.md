# T-00543 — Evidence & Audit Trail / MCP/API surface: Scaffold

## 1. Scaffold Scope
This task creates the JSON-RPC interface declarations, tool manifest registrations, and smoke test scaffold for Evidence & Audit Trail MCP tools in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. Scaffold Contents
- Registered `aios.evidence.verify`, `aios.evidence.hash`, and `aios.evidence.scan` in `tool_manifest()`.
- Scaffolded call dispatch handlers in `call_tool`.
- Created `code/aiosh-mcp/tests/test_evidence_mcp_smoke.py`.

## 3. Test Verification
```text
PASS: aios.evidence tools present in tools/list
PASS: aios.evidence.hash execution
PASS: aios.evidence.verify execution
PASS: aios.evidence.scan execution
All evidence MCP smoke tests passed successfully!
```
