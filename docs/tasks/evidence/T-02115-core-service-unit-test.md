# Core Service Unit Test Details: T-02115

- **Task**: T-02115 (PEP Decision Engine / core service: Unit Test)
- **Suite**: `aiosh-core/tests/test_pep_decision_service.rs`
- **Output**:
```
running 8 tests
test test_service_add_and_get_rule ... ok
test test_service_evaluation ... ok
test test_service_capacity_limit ... ok
test test_service_new_empty ... ok
test test_service_path_traversal_rejected ... ok
test test_service_remove_rule ... ok
test test_service_load_or_recover_corrupt ... ok
test test_service_save_and_load ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
```
- **Invariants Verified**:
  - `PEPSERV1`: In-memory thread-safe data structures
  - `PEPSERV2`: Atomic file save with atomic rename
  - `PEPSERV3`: Multi-index maintenance across subject and action
  - `PEPSERV5`: Path validation and non-destructive quarantine
  - `PEPSERV6`: Hard capacity ceiling of 5,000 rules
