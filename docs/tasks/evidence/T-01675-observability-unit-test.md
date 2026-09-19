# T-01675: Observability Unit Test

## Sub-Epic
Kernel Module Management / Observability

## Objective
Add focused automated tests for the Kernel Module Management Observability subsystem in `code/aiosh-rust/aiosh-core/tests/test_kernel_module_observability.rs`.

## Unit Test Coverage
1. **`test_ko1_empty_store_and_procfs_fallback`**:
   - Asserts graceful fallback when procfs is unavailable.
   - Validates zero-counts across memory, rules, and violations.
2. **`test_ko2_mock_procfs_module_aggregation_and_memory`**:
   - Asserts accurate aggregation of loaded modules from synthetic `/proc/modules`.
   - Asserts correct memory summation (`size_bytes`).
   - Asserts runtime state breakdown (`live`, `loading`, `unloading`).
   - Asserts reference count bucketing (`"0"`, `"1-2"`, `"3-5"`, `"6+"`).
3. **`test_ko3_rule_type_distribution_and_autoload`**:
   - Asserts categorical breakdown across modprobe rule types (`blacklist`, `alias`, `options`, `install`, `remove`, `softdep`).
   - Asserts autoload module count.
4. **`test_ko4_policy_compliance_and_prohibited_tracking`**:
   - Asserts evaluation of store rules and autoload entries against `KernelModuleSecurityPolicy`.
   - Asserts tracking of configured prohibited modules and protected modules.
5. **`test_ko5_deterministic_serialization`**:
   - Asserts roundtrip serialization and deserialization fidelity using `BTreeMap`.

## Verification
- Run via `cargo test -p aiosh-core --test test_kernel_module_observability`.
- All 5 tests pass.
