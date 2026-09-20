# T-02033: Capability Model / MCP/API Surface — Scaffold

**Task ID**: `T-02033`  
**Phase**: Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic**: Sub-Epic 4: Capability Model / MCP/API Surface  
**Status**: COMPLETED  
**Date**: 2026-09-20  

---

## 1. Scaffold Summary

The module skeleton, interfaces, and dispatch registration for the Capability Model MCP/API surface have been created:
1. **Core Service Extension**:
   - Added `load_or_create(path: &Path) -> Result<Self, String>` to `aiosh_core::capability_service::CapabilityService` in `code/aiosh-rust/aiosh-core/src/capability_service.rs`.
2. **MCP Tool Manifest**:
   - Added 7 tool definitions with JSON Schema Draft-07 input specifications to `Server::tool_manifest(&self)` in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
     - `aios.capability.list`
     - `aios.capability.get`
     - `aios.capability.issue`
     - `aios.capability.attenuate`
     - `aios.capability.revoke`
     - `aios.capability.check`
     - `aios.capability.prune`
3. **MCP Tool Dispatch Routing**:
   - Wired tool dispatch match arms in `Server::call_tool` routing through `dispatch::recorded_call`.
   - Scaffold bodies fail loudly with descriptive errors pending full implementation in `T-02034`.

---

## 2. Compilation & Verification

The project compiles cleanly with zero errors under `cargo check -p aiosh-mcp`.
All 7 tool declarations are referenced and discoverable via `tools/list`.
