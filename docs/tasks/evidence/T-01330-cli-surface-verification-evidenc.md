# T-01330: Init & Service Supervision - CLI Surface: Verification & Evidence

## Metadata
- **Task ID:** `T-01330`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Init & Service Supervision CLI Surface Verification & Evidence
- **Status:** Complete

## 1. Suite Verification Run
Executed `python tools/test_service_suites.py`:

```
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate, list, show/status, action, order)
[+] SS3 service MCP tool surface (validate)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)

PASS: service_suites criteria (SS1..SS4)
```

Executed `python code/aiosh-cli/tests/test_service_cli_smoke.py`:
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

Executed `cargo test --bin aiosh test_cmd_service_flow`:
```
running 1 test
test task_cli_tests::test_cmd_service_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.91s
```

## 2. Invariant & Feature Status
- Criteria SS1..SS4 clean PASS.
- CLI surface reachable through `aiosh service`:
  - `validate`: Invariant validation for names (`SS1`) and full specifications (`SS1..SS5`).
  - `list`: Query matrix with state, mode, pattern, limit, and store filters.
  - `show` / `status`: Metadata and status inspection.
  - `action`: Full FSM transition engine with atomic persistence.
  - Action shortcuts: `start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`.
  - `order`: Kahn's algorithm topological startup planning with cycle detection.
- Cross-substrate parity: JSON envelopes with exit codes and typed data payloads.
- Audit emission: Unconditional `classify_and_emit` logging across all branches.
- Zero known regressions across workspace.
