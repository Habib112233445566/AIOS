# T-01733: Hardware Detection — MCP/API Surface Scaffold

## Metadata
- **Task ID**: `T-01733`
- **Sub-Epic**: Sub-Epic 4: Hardware Detection MCP/API Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Scaffold Implementation
1. **Tool Schema Declarations in `tools/list`**:
   - Registered `aios.hardware.scan`, `aios.hardware.list`, `aios.hardware.get`, `aios.hardware.summary`, and `aios.hardware.verify` into `tools/list` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
   - Strictly enforced `additionalProperties: false` and parameter schemas as per invariant HM1.
2. **Dispatch Match Arms**:
   - Added match arms for all 5 tools into `call_tool` routing through `dispatch::recorded_call`.
3. **Compilation Verification**:
   - Successfully verified via `cargo check -p aiosh-mcp` with zero warnings or errors.
