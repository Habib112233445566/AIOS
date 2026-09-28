# T-02436: Sandbox Enforcement MCP/API Surface Integration

## 1. Integration Scope & Verification
This document records verification evidence for task T-02436: Sandbox Enforcement MCP/API Surface Integration.
The MCP/API tools for Sandbox Enforcement (`aios.sandbox.profiles`, `aios.sandbox.probe`, and `aios.sandbox.exec`) have been verified end-to-end within the production JSON-RPC 2.0 stdio server (`aiosh-mcp`).

## 2. Integration Verifications
1. **Tool Schema Publishing**:
   - `tools/list` publishes JSON Schema definitions for all 3 sandbox tools.
2. **Execution & Supervision**:
   - `aios.sandbox.exec` dispatches commands into `SandboxService`, verifying profile resolution, directory boundaries, and capturing process execution results.
3. **Cross-Substrate Persistence & Audit Ring Parity**:
   - Every invocation of `aios.sandbox.exec` routes through `dispatch::recorded_call`, writing an immutable row to `audit_ring` in SQLite WAL.
   - Verified via `test_mcp_sandbox_audit_persistence`.

## 3. Results
```text
============================== 8 passed in 0.74s ==============================
```
Integration smoke suite passes cleanly.
