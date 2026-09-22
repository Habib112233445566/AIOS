# Task Evidence: T-02220 (Grant Lifecycle / core service: Verification & Evidence)

## 1. Scope & Execution
Verification and milestone closure of the Grant Lifecycle Core Service Subsystem (Sub-Epic 2, tasks T-02211..T-02220).
All core service components, multi-index synchronization, attenuation containment calculus, cascade revocation, temporal/quota sweep, crash-resilient persistence, security hardening, and documentation were verified across Rust unit tests, CLI smoke tests, and MCP JSON-RPC integration suites.

---

## 2. Test Execution Output

### Rust Core Service Unit & Hardening Tests
```
> cargo test -p aiosh-core --test test_pep_grant_service --test test_pep_grant
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.01s
     Running tests\test_pep_grant.rs (target\debug\deps\test_pep_grant-7ee0dc01a1535122.exe)

running 10 tests
test test_pep_grant_action_validation ... ok
test test_pep_grant_attenuation ... ok
test test_pep_grant_fsm_transitions ... ok
test test_pep_grant_hardening_bounds ... ok
test test_pep_grant_invalid_identifier_and_scope ... ok
test test_pep_grant_store_operations_and_cascade_revocation ... ok
test test_pep_grant_store_hardening_file_limits ... ok
test test_pep_grant_temporal_and_quota_evaluation ... ok
test test_pep_grant_valid_creation_and_validation ... ok
test test_pep_grant_store_atomic_persistence ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests\test_pep_grant_service.rs (target\debug\deps\test_pep_grant_service-d0134ba36e32d643.exe)

running 12 tests
test test_pep_grant_service_attenuation ... ok
test test_pep_grant_service_cascade_revocation ... ok
test test_pep_grant_service_evaluation_and_usage ... ok
test test_pep_grant_service_hardening ... ok
test test_pep_grant_service_issue_and_query ... ok
test test_pep_grant_service_negative_attenuation_and_eval ... ok
test test_pep_grant_service_negative_capacity_and_transitions ... ok
test test_pep_grant_service_negative_path_and_file_checks ... ok
test test_pep_grant_service_scaffold_creation ... ok
test test_pep_grant_service_sweep_expired ... ok
test test_pep_grant_service_transition_and_indexes ... ok
test test_pep_grant_service_persistence ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

### Pytest Smoke Test Suite
```
> python -m pytest code/aiosh-cli/tests/test_pep_cli_smoke.py code/aiosh-mcp/tests/test_pep_decision_smoke.py
============================= test session starts =============================
platform win32 -- Python 3.14.6, pytest-9.1.1, pluggy-1.6.0
collected 16 items

code\aiosh-mcp .........                                                 [ 56%]
code\aiosh-mcp\tests\test_pep_decision_smoke.py .......                  [100%]

============================= 16 passed in 9.18s ==============================
```

### PEP CLI Smoke Test Execution
```
> python code/aiosh-cli/tests/test_pep_cli_smoke.py
PASS: aiosh pep --help
PASS: aiosh pep unknown_cmd returns 2
PASS: aiosh pep path hygiene enforcement
PASS: aiosh pep lifecycle and evaluation
PASS: aiosh pep security policy privilege boundary
PASS: aiosh pep report CLI integration
PASS: aiosh pep doc CLI integration
PASS: aiosh pep recovery & validation CLI integration
PASS: aiosh pep grant CLI integration
=== All PEP CLI tests passed ===
```

### PEP MCP Smoke Test Execution
```
> python code/aiosh-mcp/tests/test_pep_decision_smoke.py
=== PEP Decision Engine MCP Smoke Test ===
TEST: tool registration via tools/list ... OK
TEST: PEP decision evaluation ... OK
TEST: PEP MCP persistent lifecycle (status, rule_add, list, eval, remove) ... OK
TEST: PEP observability report ... OK
TEST: PEP MCP documentation tool (list, get, search) ... OK
TEST: PEP MCP recovery & validation tools (validate, recover) ... OK
TEST: PEP MCP grant lifecycle tools (list, inspect, validate, revoke) ... OK
=== All PEP Decision Engine smoke tests passed ===
```

---

## 3. Sub-Epic 2 Closure Confirmation
- [x] Invariants `GSVC1..GSVC6` fully implemented, integrated, hardened, and documented.
- [x] Zero compiler warnings across all rust workspace targets (`cargo check --bin aiosh --bin aiosh-mcp`).
- [x] 100% test pass rate across Rust unit tests (22/22) and Python integration smoke tests (16/16).
- [x] Sub-Epic 2 (Grant Lifecycle / core service) formally verified and closed.
