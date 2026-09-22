# Task Evidence: T-02236 (Grant Lifecycle / MCP/API surface: Integration)

## 1. Metadata
- **Task ID:** `T-02236`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle MCP/API Surface Integration (`code/aiosh-rust/aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-23
- **Milestone:** Sub-Epic 4: Grant Lifecycle MCP Surface (4/10) — Integration

---

## 2. Integration Summary

1. **Complete Lifecycle Integration**:
   - Integrated the full 7-tool MCP grant lifecycle suite (`aios.pep.grant.issue`, `aios.pep.grant.attenuate`, `aios.pep.grant.list`, `aios.pep.grant.inspect`, `aios.pep.grant.validate`, `aios.pep.grant.revoke`, `aios.pep.grant.sweep`) into the production MCP server stdio dispatcher.
   - Updated baseline smoke test (`code/aiosh-mcp/tests/test_pep_decision_smoke.py`) to eliminate mock file writing and exercise root issuance directly via `aios.pep.grant.issue`.
2. **Cross-Substrate Parity & Discoverability**:
   - Verified that `tools/list` enumerates all grant lifecycle tools with compliant JSON Schema definitions.
   - Verified cross-substrate storage parity on canonical JSON files using atomic write-temp-and-rename semantics.
   - Guaranteed immutable cryptographic audit recording via `dispatch::recorded_call` appending into SQLite `$AIOSH_HOME/audit.db`.

---

## 3. Verification Output

### 3.1 Integrated Smoke Test Runner
```text
> python code/aiosh-mcp/tests/test_pep_decision_smoke.py
=== PEP Decision Engine MCP Smoke Test ===
TEST: tool registration via tools/list ... OK
TEST: PEP decision evaluation ... OK
TEST: PEP MCP persistent lifecycle (status, rule_add, list, eval, remove) ... OK
TEST: PEP observability report ... OK
TEST: PEP MCP documentation tool (list, get, search) ... OK
TEST: PEP MCP recovery & validation tools (validate, recover) ... OK
TEST: PEP MCP grant lifecycle tools (issue, list, inspect, validate, attenuate, sweep, revoke) ... OK
=== All PEP Decision Engine smoke tests passed ===
```

### 3.2 Dedicated MCP Grant Lifecycle Suite
```text
> python code/aiosh-mcp/tests/test_pep_grant_mcp.py
=== All MCP Grant Unit Tests Passed Successfully ===
```

---

## 4. Acceptance Confirmation
- [x] Feature reachable through production MCP interface over JSON-RPC 2.0 stdio.
- [x] Integration smoke passes end-to-end with zero regressions.
- [x] All 7 grant lifecycle tools discoverable in manifest.
