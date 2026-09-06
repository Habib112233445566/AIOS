# T-01340: Init & Service Supervision - MCP/API Surface: Verification & Evidence

## Metadata
- **Task ID:** `T-01340`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Init & Service Supervision MCP / API Surface Verification & Evidence
- **Status:** Complete (Milestone Closed)

---

## 1. Test Suite Verification Runs

### 1.1 Master Service Subsystem Matrix (`tools/test_service_suites.py`)
```
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate, list, show/status, action, order)
[+] SS3 service MCP tool surface (validate, list, get, action, order)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)

PASS: service_suites criteria (SS1..SS4)
```

### 1.2 Dedicated MCP Smoke Test (`code/aiosh-mcp/tests/test_service_mcp_smoke.py`)
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

### 1.3 Rust In-Process Unit Test (`aiosh-mcp`)
Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml --bin aiosh-mcp test_mcp_service_tools`
Output:
```
running 1 test
test tests::test_mcp_service_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; finished in 0.11s
```

### 1.4 CLI Smoke Test Suite (`code/aiosh-cli/tests/test_service_cli_smoke.py`)
```
PASS: aiosh service --help
PASS: aiosh service unknown_cmd returns 2
PASS: aiosh service validate (name and json)
PASS: aiosh service list (prose, json, filters)
PASS: aiosh service show and status (prose, json, not found)
PASS: aiosh service action and direct shortcuts (start/stop/restart/reload)
PASS: aiosh service order (valid topological plan and negative tests)

ALL SERVICE CLI SMOKE TESTS PASSED!
```

### 1.5 Repository Documentation Linter (`tools/check_task_docs.py`)
```
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```

---

## 2. Invariant & Feature Status
- Criteria SS1..SS4 clean PASS.
- Full MCP surface operational:
  - `aios.service.validate`: SS1 name syntax & SS1..SS5 specification validation.
  - `aios.service.list`: Filtered service catalog discovery with state/mode/pattern filters.
  - `aios.service.get`: Specification and live runtime status inspection.
  - `aios.service.action`: State machine lifecycle engine with atomic persistence.
  - `aios.service.order`: Topological startup execution planning using Kahn's algorithm.
- Cross-substrate parity maintained across Rust core, CLI commands, and MCP tools.
- Audit emission: Unconditional `classify_and_emit` logging across all execution paths.
- Updated `task_plan.md` and `progress.md` closing milestone `Init & Service Supervision / MCP API surface` (T-01331..T-01340).
- Zero known regressions across workspace.
