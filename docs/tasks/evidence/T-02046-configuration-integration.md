# Task Evidence: T-02046 (Capability Model / configuration: Integration)

## Task Information
- **Task ID**: T-02046
- **Title**: Capability Model / configuration: Integration
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Integration & Smoke Testing
1. **Integration Test Suite**: `code/aiosh-mcp/tests/test_capability_config_smoke.py`
   - Target Binary: `code/aiosh-rust/target/debug/aiosh-mcp.exe`

2. **Integration Verification Areas**:
   - **Schema Parity & Defaults**:
     - Verified default configuration keys and values (`version`, `store_path`, `max_store_bytes`, `max_capabilities`, `enforce_strict_monotonic`, `auto_prune_on_load`).
   - **Runtime Environment Override & Persistence**:
     - Set `AIOS_CAPABILITY_STORE_PATH`, `AIOS_CAPABILITY_MAX_CAPABILITIES`, `AIOS_CAPABILITY_MAX_STORE_BYTES`.
     - Tested full capability lifecycle (`issue`, `get`, `check`, `revoke`) against the custom store path and environment settings via MCP JSON-RPC.
   - **Path Hygiene & Boundary Validation**:
     - Verified path traversal (`../`) injection rejection.
     - Verified ASCII control character (`\0`) injection rejection.

3. **Execution Results**:
   - `TEST: CapabilityConfig schema parity and default values ... OK`
   - `TEST: Runtime capability operations with configured environment ... OK`
   - `TEST: Path hygiene and boundary validation ... OK`
   - `ALL TESTS PASSED`
