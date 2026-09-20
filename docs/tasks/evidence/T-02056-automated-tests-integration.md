# Task Evidence: T-02056 (Capability Model / automated tests: Integration)

## Task Information
- **Task ID**: T-02056
- **Title**: Capability Model / automated tests: Integration
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 6: Automated Tests
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Integration & Smoke Testing
1. **Integration Test Suite**: `code/aiosh-mcp/tests/test_capability_automated_smoke.py`
   - Target Binary: `code/aiosh-rust/target/debug/aiosh-mcp.exe`

2. **Test Scenarios Verified**:
   - **Multi-tier Capability Attenuation & Cascade Revocation**:
     - 4-tier capability hierarchy created over MCP JSON-RPC (`aios.capability.issue` -> `aios.capability.attenuate` Tier 1 -> Tier 2 -> Tier 3).
     - Verified granular rights checking at Tier 3 (Read granted, Write denied).
     - Revocation at Tier 1 verified to cascade transitively to Tier 1, 2, and 3.
     - Confirmed Root (Tier 0) remained active and unaffected.
   - **Quota Exhaustion**:
     - Root capability with `max_invocations: 2` consumed via `aios.capability.check` with `consume: true`.
     - First two invocations granted; third invocation denied upon quota exhaustion.
   - **Fault Injection & Boundary Protection**:
     - Path traversal (`../evil.json`) in `store_path` rejected with structured error.
     - Non-json extension (`store.txt`) rejected with structured error.

3. **Execution Results**:
   - `TEST: Multi-tier capability attenuation and cascade revocation ... OK`
   - `TEST: Invocation quota exhaustion over MCP ... OK`
   - `TEST: Fault injection and path protection ... OK`
   - `ALL AUTOMATED TESTS PASSED`
