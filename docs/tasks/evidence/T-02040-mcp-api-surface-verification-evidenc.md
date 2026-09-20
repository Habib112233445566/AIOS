# Task Evidence: T-02040 (Capability Model / MCP/API surface: Verification & Evidence)

## Task Information
- **Task ID**: T-02040
- **Title**: Capability Model / MCP/API surface: Verification & Evidence
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 4: MCP / API Surface Integration (Formal Closure)
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Verification & Evidence
1. **Scope of Verification**:
   - Verification of the complete `aios.capability.*` MCP tool suite across both unit and integration test levels.
   - Formal closure of Sub-Epic 4: MCP / API Surface Integration.

2. **Automated Unit Tests**:
   - Command: `cargo test -p aiosh-mcp test_capability_mcp_tools` in `code/aiosh-rust`
   - Result: 1 passed; 0 failed; finished in 0.13s.
   - Validated:
     - Tool registration in `list_tools()`
     - Issue root capability
     - Attenuate capability
     - Check capability (valid, expired, wrong right)
     - Revoke capability and cascade revocation verification
     - Prune expired capabilities

3. **End-to-End Integration / Smoke Tests**:
   - Command: `python code/aiosh-mcp/tests/test_capability_mcp_smoke.py`
   - Target Binary: `code/aiosh-rust/target/debug/aiosh-mcp.exe`
   - Result:
     - `TEST: tool registration via tools/list ... OK`
     - `TEST: capability full lifecycle over MCP JSON-RPC ... OK`
     - `ALL TESTS PASSED`

4. **Security & Invariant Validation**:
   - Boundary checks on input strings (length limits, control character rejection).
   - Bounds validation on integer fields (`max_invocations`, `quota_bytes`, `expires_in_secs`).
   - JSON-RPC error codes properly emitted for violations.
   - Formal closure of Sub-Epic 4 confirmed.
