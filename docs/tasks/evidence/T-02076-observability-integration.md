# Evidence: T-02076 - observability: Integration

## Task Overview
- **Task ID**: `T-02076`
- **Sub-Epic**: Sub-Epic 8: Observability (`T-02071`..`T-02080`)
- **Phase**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Goal**: Integrate Capability Observability into the MCP JSON-RPC protocol surface and verify cross-surface functionality.

## Integration Details
1. **MCP Tool Registration (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
   - Added `aios.capability.observability` to `list_tools()` with schema accepting optional `store_path` and `grant_id`.
   - Wired `aios.capability.observability` in `call_tool()` via `dispatch::recorded_call`, preserving PEP gating and audit ring logging.
2. **Integration Smoke Suite (`code/aiosh-mcp/tests/test_capability_observability_smoke.py`)**:
   - `test_tool_registration`: Confirms tool is advertised in `tools/list`.
   - `test_observability_lifecycle`:
     - Generates report on empty store (`total_capabilities: 0`, `is_healthy: true`).
     - Issues root capability and attenuates child capability.
     - Confirms metrics update (`total_capabilities: 2`, `root_capabilities: 1`, `attenuated_capabilities: 1`, `max_derivation_depth: 1`, `active_capabilities: 2`).
     - Revokes root capability.
     - Confirms cascade revocation reflects in telemetry (`total_capabilities: 2`, `revoked_capabilities: 2`, `active_capabilities: 0`).
3. **Execution Output**:
   ```text
   Running Capability Observability Integration Smoke against binary: C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\target\debug\aiosh-mcp.exe
   TEST: tool registration via tools/list ... OK
   TEST: capability observability report lifecycle over MCP ... OK
   ALL OBSERVABILITY INTEGRATION TESTS PASSED
   ```
4. **Regression Verification**:
   - `cargo test --test test_capability_observability`: 5/5 unit tests passing in 0.00s.
   - `cargo test --test test_capability_policy`: 9/9 unit tests passing in 0.01s.
