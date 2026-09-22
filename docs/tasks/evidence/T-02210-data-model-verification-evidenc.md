# Task Evidence: T-02210 (Grant Lifecycle / data model: Verification & Evidence)

## 1. Scope & Execution
Verification and milestone closure of the Grant Lifecycle Data Model (Sub-Epic 1, tasks T-02201..T-02210).
Executed full verification across unit tests, CLI smoke tests, and MCP smoke tests.

---

## 2. Test Execution Output

### Rust Unit & Hardening Test Suite
```
> cargo test -p aiosh-core --test test_pep_grant
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.85s
     Running tests\test_pep_grant.rs (target\debug\deps\test_pep_grant-7ee0dc01a1535122.exe)

running 10 tests
test test_pep_grant_attenuation ... ok
test test_pep_grant_action_validation ... ok
test test_pep_grant_fsm_transitions ... ok
test test_pep_grant_hardening_bounds ... ok
test test_pep_grant_invalid_identifier_and_scope ... ok
test test_pep_grant_store_operations_and_cascade_revocation ... ok
test test_pep_grant_store_hardening_file_limits ... ok
test test_pep_grant_temporal_and_quota_evaluation ... ok
test test_pep_grant_valid_creation_and_validation ... ok
test test_pep_grant_store_atomic_persistence ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### PEP CLI Smoke Test Suite
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

### PEP MCP Smoke Test Suite
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

## 3. Milestone Completion Confirmation
- [x] Invariants `PEPGRANT1..PEPGRANT6` verified and enforced.
- [x] 100% pass rate across unit tests and end-to-end smoke suites.
- [x] Documentation updated in `docs/pep_decision_engine.md` Section 15.
- [x] `progress.md` updated recording Sub-Epic 1 milestone closure.
- [x] Sub-Epic 1 (Data Model: T-02201..T-02210) formally closed. Ready to launch Sub-Epic 2 (Core Service: T-02211..T-02220).
