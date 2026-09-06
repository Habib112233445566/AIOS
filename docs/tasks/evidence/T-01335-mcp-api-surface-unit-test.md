# T-01335: Init & Service Supervision - MCP/API Surface: Unit Test

## Metadata
- **Task ID:** `T-01335`
- **Subsystem:** `code/aiosh-mcp`
- **Component:** Init & Service Supervision MCP / API Surface Unit Tests
- **Status:** Complete (Unit Test)

---

## 1. Test Suite Overview
Created a dedicated, standalone integration and unit smoke test suite:
`code/aiosh-mcp/tests/test_service_mcp_smoke.py`

This suite exercises the full JSON-RPC 2.0 stdio protocol against the compiled `aiosh-mcp` server binary, verifying all five service supervision MCP tools across valid inputs, invalid inputs, boundary values, state machine invariants, persistence, and failure modes.

---

## 2. Test Cases & Coverage Matrix

| Test Function | Tool Tested | Scenarios Covered |
|:--------------|:------------|:------------------|
| `test_manifest` | `tools/list` | Verifies presence of `aios.service.validate`, `aios.service.list`, `aios.service.get`, `aios.service.action`, and `aios.service.order`. |
| `test_validate` | `aios.service.validate` | 1. Valid service name (`auditd.service`).<br>2. Invalid service name with illegal slash (`invalid/service`).<br>3. Control character in name (`\x07`).<br>4. Full valid `ServiceSpec` object.<br>5. Invalid `ServiceSpec` with self-dependency cycle.<br>6. Missing arguments rejection. |
| `test_list` | `aios.service.list` | 1. Default unfiltered service catalog query (count >= 6).<br>2. Substring filter by pattern (`audit`).<br>3. Exact state filter (`active`).<br>4. Invalid state enum rejection.<br>5. Control character bounds check in pattern. |
| `test_get` | `aios.service.get` | 1. Query registered service (`auditd.service`) returning spec and live status.<br>2. Non-existent service (`nonexistent.service`) error.<br>3. Missing required parameter rejection.<br>4. Control character rejection. |
| `test_action_and_persistence` | `aios.service.action` | 1. `stop` transition with state change to `inactive`.<br>2. Verification of disk persistence via custom temporary store path.<br>3. `restart` action verifying state transition to `active`.<br>4. Rejection of `mask` when service is active (enforcing prior stop).<br>5. `mask` action on inactive service.<br>6. Rejection of `start` on a masked service.<br>7. `unmask` action.<br>8. Rejection of invalid action verb. |
| `test_order` | `aios.service.order` | 1. Topological startup ordering for `aios-securityd.service` verifying dependency resolution (`auditd.service`, `dbus.service`).<br>2. Missing target rejection.<br>3. Missing required arguments rejection. |

---

## 3. Test Execution & Output
Command: `python code/aiosh-mcp/tests/test_service_mcp_smoke.py`
Output:
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
Zero regressions, all positive and negative assertions validated.
