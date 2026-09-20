# Evidence: T-02066 - integration

## Task Overview
- **Task ID**: `T-02066`
- **Sub-Epic**: Sub-Epic 7: Security Policy (`T-02061`..`T-02070`)
- **Phase**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Action**: End-to-end integration of Capability Security Policy over the MCP JSON-RPC protocol.

## Summary
- Integrated `CapabilitySecurityPolicy` into `CapabilityService` and exposed via MCP tools `aios.capability.issue` and `aios.capability.attenuate`.
- Authored and verified `code/aiosh-mcp/tests/test_capability_policy_smoke.py` (3 tests, all passed).
- Re-verified automated capability test suite (8 tests, all passed).
