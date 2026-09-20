# T-02116: Core Service Integration — PEP Decision Engine

## Overview
- **Task ID**: `T-02116`
- **Sub-Epic**: 2 (Core Service)
- **Component**: PEP Decision Engine Core Service (`PepDecisionService` & `aiosh-mcp`)
- **Status**: Completed

## Integration Details
1. **MCP Surface Integration**:
   - `aios.pep.evaluate` registered in `code/aiosh-rust/aiosh-mcp/src/main.rs` under `tools()` with complete JSON schema:
     - Properties: `subject`, `resource`, `action`, `algorithm`, `rules`, `grant_id`.
     - Required: `subject`, `resource`, `action`.
   - Dispatch handler in `Server::call_tool` routes calls through `dispatch::recorded_call`, validating inputs, evaluating rules, and enforcing `PEPDEC1..PEPDEC6` invariants.
2. **Audit Gating & Parity**:
   - Every evaluation emits an audit row into the tamper-evident SQLite audit ring.
   - Cross-substrate parity validated: canonical JSON serialization matches schema for rule storage and decision records.
3. **Smoke Test Execution**:
   - Executed `code/aiosh-mcp/tests/test_pep_decision_smoke.py`.
   - Verified `aios.pep.evaluate` discovery in `tools/list` and end-to-end evaluation.
   - Result: PASS.
