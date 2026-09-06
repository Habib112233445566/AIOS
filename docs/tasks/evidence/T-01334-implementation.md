# T-01334: Init & Service Supervision - MCP/API Surface: Implementation

## Metadata
- **Task ID:** `T-01334`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Init & Service Supervision MCP / API Surface
- **Status:** Complete (Implementation)

---

## 1. Summary of Changes & Implementation Details
This task verifies and documents the complete working implementation of the Init & Service Supervision MCP API surface in `code/aiosh-rust/aiosh-mcp`.

### 1.1 Implemented Tools
1. **`aios.service.validate`**:
   - Accepts either `name` or full `spec`.
   - Validates name syntax (`SS1`) or checks full specification invariants (`SS1..SS5`).
   - Returns structured validation report `{ ok: true, tool: "aios.service.validate", valid: true, ... }`.
2. **`aios.service.list`**:
   - Parses optional filter query: substring `pattern`, `state`, `startup_mode`, `limit`, and custom `store_path`.
   - Rejects strings exceeding length limits or containing control characters.
   - Queries `ServiceStore` and returns list with total matching count.
3. **`aios.service.get`**:
   - Validates required `name` parameter.
   - Loads service specification and live runtime status from `ServiceStore`.
   - Returns both spec and status, or a clear not-found error.
4. **`aios.service.action`**:
   - Validates `name` and `action` parameters.
   - Parses actions (`start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`).
   - Enforces state machine invariants (e.g. masked services cannot be activated).
   - Atomically persists changes to disk via `store.save_to_path` if custom `store_path` is given.
5. **`aios.service.order`**:
   - Computes topological activation order for target service using Kahn's algorithm in `plan_service_order`.
   - Traverses dependency graph, detects cycles, and outputs ordered list of service names.

### 1.2 Policy Enforcement & Audit Trail Integration
- All five service tools execute through `dispatch::recorded_call`.
- Each call is subject to PEP authorization checking (`check_authorization`).
- Each call emits an audit row to `self.ring` through `classify_and_emit`.
- Consequential mutations record target service names and action verbs.

---

## 2. Verification & Test Evidence
Executed targeted test:
```
cargo test --bin aiosh-mcp test_mcp_service_tools
```
Output:
```
running 1 test
test tests::test_mcp_service_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; finished in 0.11s
```

Full service test suites:
```
python tools/test_service_suites.py
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate, list, show/status, action, order)
[+] SS3 service MCP tool surface (validate)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)

PASS: service_suites criteria (SS1..SS4)
```
Zero regressions observed.
