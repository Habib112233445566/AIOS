# T-01215: Package Management - Core Service: Unit Test

## Metadata
- **Task ID:** `T-01215`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package_service`
- **Component:** Package Management Core Service Automated Tests
- **Status:** Complete

## 1. Test Suite Deliverables
Implemented standalone integration test suite in [test_package_service.rs](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/tests/test_package_service.rs) covering happy and negative paths across invariants CS1..CS5:
1. `test_package_store_seeding_and_lookup`: Confirms store seeds 8 canonical reference packages across Debian and Alpine and looks up existing/missing packages.
2. `test_package_store_cs1_uniqueness_and_lifecycle`: Confirms package registration, duplicate rejection enforcing CS1 uniqueness, and unregistration.
3. `test_package_store_query_matrix`: Exercises query engine filtering across Debian (`Deb`), Alpine (`Apk`), states (`Installed`, `Available`), substring pattern matching, and limit bounds.
4. `test_package_store_cs2_determinism`: Asserts that planning identical actions against unchanged state produces byte-identical transaction plans.
5. `test_package_store_cs3_dependency_closure`: Confirms install attempts with missing dependencies are rejected with CS3 errors, while batched dependency satisfies closure.
6. `test_package_store_cs4_delta_arithmetic_and_tamper_detection`: Confirms exact calculation of size delta on install/remove, and asserts tampered transactions are rejected on execution.
7. `test_package_store_dry_run_vs_actual_execution`: Confirms dry-run executes without mutating package state in store, while live transaction applies state transitions.
8. `test_package_store_cs5_persistence_and_bounds`: Confirms atomic persistence roundtrip and enforces rejection of files exceeding the 10 MiB limit.

## 2. Test Execution Output
```
running 8 tests
test test_package_store_cs1_uniqueness_and_lifecycle ... ok
test test_package_store_cs3_dependency_closure ... ok
test test_package_store_cs2_determinism ... ok
test test_package_store_cs4_delta_arithmetic_and_tamper_detection ... ok
test test_package_store_dry_run_vs_actual_execution ... ok
test test_package_store_query_matrix ... ok
test test_package_store_seeding_and_lookup ... ok
test test_package_store_cs5_persistence_and_bounds ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```
