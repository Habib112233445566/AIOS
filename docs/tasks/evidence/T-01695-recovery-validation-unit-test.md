# T-01695: Kernel Module Management Recovery & Validation Unit Test

## Sub-Epic
Kernel Module Management / Recovery & Validation (T-01695)

## Objective
Add focused automated tests covering valid input, invalid input, boundary values, corruption recovery, and conflict resolution for the Kernel Module Management recovery & validation subsystem.

## Test Battery Overview
Test suite implemented at `code/aiosh-rust/aiosh-core/tests/test_kernel_module_recovery.rs`:

1. **`test_kr1_kr2_kr3_healthy_store_validation`**:
   - Asserts KR1 (`valid_rules + invalid_rules == total_rules`).
   - Asserts KR2 (`valid_autoload + invalid_autoload == total_autoload`).
   - Asserts KR3 (`healthy == (errors.is_empty() && invalid_rules == 0 && invalid_autoload == 0)`).
   - Validates that a well-formed store produces a completely green report.

2. **`test_kr4_conflict_detection_and_resolution`**:
   - Tests detection of a store containing conflicting autoload and blacklist directives (KM3 / KR4).
   - Verifies `check_store_file` flags the conflict and marks store unhealthy.
   - Verifies `recover_store_file` creates a quarantine backup, resolves the conflict by removing the conflicting autoload entry, and leaves the store in a clean, healthy state.

3. **`test_kr5_unparseable_json_quarantine_and_reinitialization`**:
   - Tests severely corrupt / truncated JSON files.
   - Verifies non-destructive quarantine (`.corrupt.<timestamp>.bak`) preserves the exact damaged content.
   - Verifies reinitialization of a valid default store.

4. **`test_kr6_partial_corruption_repair_and_backup`**:
   - Injects command injection strings (`bad;rm -rf /`) and invalid autoload module names.
   - Verifies recovery purges malicious/invalid rules while retaining valid rules and autoload targets.

5. **`test_non_existent_file_check_and_recovery`**:
   - Verifies `check_store_file` handles missing file gracefully.
   - Verifies `recover_store_file` provisions a fresh default store.

## Execution & Results
Ran: `cargo test -p aiosh-core --test test_kernel_module_recovery`
```
running 5 tests
test test_kr1_kr2_kr3_healthy_store_validation ... ok
test test_non_existent_file_check_and_recovery ... ok
test test_kr4_conflict_detection_and_resolution ... ok
test test_kr5_unparseable_json_quarantine_and_reinitialization ... ok
test test_kr6_partial_corruption_repair_and_backup ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```
Plus regression run of all 47 kernel module tests in `aiosh-core` — 100% green.
