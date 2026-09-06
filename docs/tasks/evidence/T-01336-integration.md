# T-01336: Init & Service Supervision - MCP/API Surface: Integration

## Metadata
- **Task ID:** `T-01336`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Init & Service Supervision MCP / API Surface Integration
- **Status:** Complete (Integration)

---

## 1. System Integration Overview
The Init & Service Supervision MCP API surface integrates directly into the platform runtime:
1. **MCP Discovery & Registration**:
   - Registered in `Server::tool_manifest` as standard JSON-RPC 2.0 tools with schema validation.
   - Tools are immediately discoverable via `tools/list` by MCP clients, language model agents, and orchestration engines.
2. **Dispatch & Policy Enforcement (PEP)**:
   - Tool calls route through `dispatch::recorded_call`.
   - Gated by Policy Enforcement Point capability checks (`grant_id`), with failure rejection before any side effects occur.
3. **Audit Ring Non-Repudiation (ADR-0035)**:
   - Invocations emit non-repudiable audit events into `AuditRing` via `classify_and_emit`.
   - Consequential state changes (e.g. `aios.service.action`) log target entity identity and action verbs.
4. **Cross-Substrate State Parity**:
   - MCP tools (`aios.service.*`) and CLI commands (`aiosh service *`) operate against identical underlying state representations (`aiosh_core::service::ServiceSpec` and `ServiceStatus`).
   - Mutations made via MCP actions are reflected in subsequent CLI inspections and vice-versa when targeting the same store file.

---

## 2. Verification Suite Results

### 2.1 Subsystem Suite Runner (`tools/test_service_suites.py`)
```
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate, list, show/status, action, order)
[+] SS3 service MCP tool surface (validate, list, get, action, order)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)

PASS: service_suites criteria (SS1..SS4)
```

### 2.2 Dedicated MCP Smoke Suite (`code/aiosh-mcp/tests/test_service_mcp_smoke.py`)
```
=== RUNNING SERVICE MCP SMOKE TESTS ===
PASS: test_manifest (all 5 service tools registered)
PASS: test_validate (positive, negative, and boundary cases)
PASS: test_list (filtering, count, invalid enum)
PASS: test_get (found, not found, validation error)
PASS: test_action_and_persistence (lifecycle transitions, masked invariant, atomic save)
PASS: test_order (topological ordering, missing targets, error paths)

ALL SERVICE MCP SMOKE TESTS PASSED!
```

---

## 3. Integration Status & Parity
- Feature is fully reachable through its production MCP stdio JSON-RPC interface.
- Complete parity established between Rust in-memory service models, CLI commands, and MCP tools.
- Zero regressions across existing test suites.
