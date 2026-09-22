# Task Evidence: T-02214 (Grant Lifecycle / core service: Implementation)

## 1. Scope & Execution
Implemented the complete, production-grade `PepGrantService` in `code/aiosh-rust/aiosh-core/src/pep_grant_service.rs` adhering to invariants `GSVC1..GSVC6`:
- **`GSVC1` (Multi-Index Consistency)**: Primary `HashMap<String, PepGrant>` with secondary multi-maps `by_subject`, `by_parent`, and `by_state`. All indexes atomically updated upon grant issuance, state transitions, usage metering, and revocation. Includes `rebuild_indexes()`.
- **`GSVC2` (Authoritative Issuance & FSM Transitions)**: Validates incoming grants, bounds store capacity (`MAX_GRANTS_IN_SERVICE = 5000`), and executes legal FSM state transitions (`Requested -> Active/Revoked`, `Active -> Suspended/Revoked/Expired`, `Suspended -> Active/Revoked/Expired`, terminal sinks `Revoked` and `Expired`).
- **`GSVC3` (Attenuation & Delegation Engine)**: `attenuate_grant()` validates parent existence, active state, `CapabilityRight::Delegate` right, and `max_delegation_depth > 0`. Asserts child rights are a subset of parent rights. Derives, validates, and registers child grant with `parent_grant_id` link.
- **`GSVC4` (Action Evaluation & Quota Metering)**: `evaluate_grant()` validates requester subject, required right, temporal validity window, and quota exhaustion. `record_grant_usage()` meters invocations and bytes, automatically transitioning exhausted grants to `Expired` and updating indices.
- **`GSVC5` (Cascade Revocation Engine)**: `revoke_grant()` revokes target grant. When `cascade == true`, computes transitive closure across `by_parent` DAG and revokes all descendants atomically with audit reasons and timestamps.
- **`GSVC6` (Expiration Sweep & Atomic Persistence)**: `sweep_expired()` queries active/suspended grants and transitions past-expiry grants to `Expired`. `save_to_path()`, `load_from_path()`, and `sync()` enforce atomic staging (`.tmp.<pid>.<nonce>`), path validation, and a 10 MiB payload ceiling.

---

## 2. Test Execution & Verification

### Targeted Unit Test Suite (`test_pep_grant_service.rs`)
```
> cargo test -p aiosh-core --test test_pep_grant_service
running 8 tests
test test_pep_grant_service_scaffold_creation ... ok
test test_pep_grant_service_issue_and_query ... ok
test test_pep_grant_service_transition_and_indexes ... ok
test test_pep_grant_service_attenuation ... ok
test test_pep_grant_service_evaluation_and_usage ... ok
test test_pep_grant_service_cascade_revocation ... ok
test test_pep_grant_service_sweep_expired ... ok
test test_pep_grant_service_persistence ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

### Regression Verification
- **PEP CLI Smoke Suite**: `python code/aiosh-cli/tests/test_pep_cli_smoke.py` -> 9/9 PASS.
- **PEP MCP Smoke Suite**: `python code/aiosh-mcp/tests/test_pep_decision_smoke.py` -> 7/7 PASS.

---

## 3. Acceptance Confirmation
- [x] Targeted unit test suite passes 100% (8/8 tests).
- [x] Zero regressions across all CLI and MCP smoke suites.
- [x] Consequential mutations write immutable audit rows via recorded dispatch.
