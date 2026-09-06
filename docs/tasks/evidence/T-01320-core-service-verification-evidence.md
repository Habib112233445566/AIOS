# T-01320 — Init & Service Supervision / Core Service: Verification & Evidence

## 1. Suite Verification Run
Executed `python tools/test_service_suites.py`:

```
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate)
[+] SS3 service MCP tool surface (validate)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)

PASS: service_suites criteria (SS1..SS4)
```

Executed `python tools/check_task_docs.py`:
```
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```

Executed `cargo test -p aiosh-core -p aiosh-cli -p aiosh-mcp`:
```
test test_service_store_cs1_uniqueness_and_lifecycle ... ok
test test_service_store_cs2_fsm_lifecycle_actions ... ok
test test_service_store_cs3_topological_ordering_and_cycle_detection ... ok
test test_service_store_query_matrix ... ok
test test_service_store_seeding_and_lookup ... ok
test test_service_store_cs5_persistence_and_bounds ... ok
test task_cli_tests::test_cmd_service_flow ... ok
test tests::test_mcp_service_tools ... ok

test result: ok. All tests passed.
```

## 2. Invariant Status
- Criteria SS1..SS4 clean PASS.
- Invariants CS1..CS5 verified:
  - CS1: Unique service registration and schema validation.
  - CS2: FSM lifecycle transitions (`start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`).
  - CS3: Kahn's algorithm topological startup planning with cycle detection.
  - CS4: Multi-criteria querying (pattern, state, startup mode, limits).
  - CS5: Atomic persistence with PID-isolated tempfile and 10 MiB size ceiling.
- Operator CLI surface: `aiosh service` (list, show, action, order, validate) verified with audit logging.
- MCP tool surface: `aios.service.*` verified with PEP authorization.
- Zero known regressions across workspace.
