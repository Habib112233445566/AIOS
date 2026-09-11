# T-01375: Init & Service Supervision - Observability: Unit Test

## Metadata
- **Task ID:** `T-01375`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Observability
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Scope & Objective
Create and execute focused automated unit and integration tests for `ServiceObservabilityReport` in `code/aiosh-rust/aiosh-core/tests/test_service_observability.rs` validating all positive, negative, and boundary scenarios for criteria `SO1..SO6`.

---

## 2. Test Cases Implemented
1. `test_so1_inventory_completeness_and_empty_store`:
   - Validates empty store returns `total_services == 0`, empty categorical breakdowns, and zeroed counters.
   - Validates default store maintains strict mathematical conservation:
     $\sum \text{state} = \sum \text{mode} = \sum \text{type} = \sum \text{policy} = \sum \text{deps} = \text{total\_services}$.
2. `test_so2_categorical_distributions`:
   - Registers services across multiple types (`simple`, `forking`, `oneshot`, `notify`), modes (`enabled`, `disabled`, `masked`, `static`), and restart policies (`always`, `on_failure`, `no`).
   - Verifies breakdown maps correctly record exact counts with sorted keys.
3. `test_so3_health_and_restart_telemetry`:
   - Validates healthy vs. unhealthy service accounting.
   - Asserts failed service tracking (`failed_services` list) and process restart aggregation (`total_restarts`).
4. `test_so4_dependency_distribution_histogram`:
   - Evaluates dependency partitioning across buckets `"0"`, `"1-2"`, `"3-5"`, and `"6+"`.
   - Asserts each bucket receives the expected number of services.
5. `test_so5_security_policy_compliance`:
   - Evaluates store containing compliant services and prohibited daemons (`telnet.service`).
   - Validates `policy_compliant_count`, `policy_violations_count`, and `prohibited_services_found`.
6. `test_so6_serialization_and_string_helpers`:
   - Verifies JSON roundtrip serialization and deserialization.
   - Asserts canonical string mappings (`service_type_to_str`, `service_state_to_str`, `startup_mode_to_str`, `restart_policy_to_str`).
7. `test_so7_hardening_and_path_boundaries`:
   - Validates `generate_from_paths` with `None`, non-existent files, control character paths, and oversized paths (> 1024 chars).
   - Verifies real file roundtrip with temporary store and policy files.

---

## 3. Execution & Verification Output
```text
running 7 tests
test test_so2_categorical_distributions ... ok
test test_so3_health_and_restart_telemetry ... ok
test test_so4_dependency_distribution_histogram ... ok
test test_so1_inventory_completeness_and_empty_store ... ok
test test_so5_security_policy_compliance ... ok
test test_so6_serialization_and_string_helpers ... ok
test test_so7_hardening_and_path_boundaries ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```
