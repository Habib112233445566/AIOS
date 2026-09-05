# T-01278: Package Management / Observability - Hardening

## Overview
Task `T-01278` hardens the package management observability and telemetry reporting subsystem (`PackageObservabilityReport`, `aiosh package stats`, and `aios.package.stats`):

1. **Input Validation & Path Hardening**:
   - `store_path` and `config_path` parameters are constrained to $\le 1024$ characters.
   - ASCII control characters are rejected immediately across CLI, MCP, and core APIs (`PackageObservabilityReport::generate_from_paths`).
   - Oversized paths (>1024 characters) produce explicit `INVALID_ARGUMENT` or path error envelopes.

2. **Explicit Error Envelopes & Audit Guarantees**:
   - In `aiosh package stats`, any path validation failure, store loading failure (`LOAD_STORE_FAILED`), or configuration/policy loading failure (`LOAD_POLICY_FAILED`) is logged to `audit.db` via `classify_and_emit` before returning non-zero exit codes (1 or 2). Silent fallbacks on invalid explicit paths are strictly prohibited.
   - In `aios.package.stats` (MCP), error paths return `{"ok": false, "error": "..."}` and are recorded by `dispatch::recorded_call` into the SQLite WAL ring buffer, preserving auditability and non-repudiation on failure.

3. **Resource Bounds & Arithmetic Safety**:
   - File size caps: `PackageStore::load_from_path` limits input payloads to 100 MiB; `PackageSecurityPolicy::from_file` limits configuration streams to 64 KiB (`MAX_POLICY_FILE_BYTES`).
   - Integer overflow protection: Memory footprint summation uses `u64::saturating_add`.
   - Division-by-zero protection: Average package calculation explicitly checks `total_packages > 0`.
   - Bounded histograms: Categorical dependency distribution buckets (`"0"`, `"1-5"`, `"6-10"`, `"11+"`) are bounded to $O(1)$ memory.

4. **Resource Cleanup**:
   - Temporary file handling in automated tests cleans up all scratch and test artifacts using explicit cleanup semantics.
   - Database handles in audit subsystems utilize WAL mode with automatic connection closing and zero resource leaks.

---

## Test Verification Output

### 1. `aiosh-core` Unit Suite (`test_package_observability.rs`)
```
running 7 tests
test test_po3_footprint_and_capacity_telemetry ... ok
test test_po4_dependency_distribution_histogram ... ok
test test_po2_state_format_arch_breakdown_distributions ... ok
test test_po1_inventory_completeness_and_empty_store ... ok
test test_po5_policy_compliance_and_prohibited_package_detection ... ok
test test_po6_serialization_and_negative_boundary_matrix ... ok
test test_po7_hardening_and_path_boundaries ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

### 2. `aiosh-cli` Unit Suite (`test_cmd_package_flow`)
```
running 1 test
test task_cli_tests::test_cmd_package_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 1.05s
```

### 3. `aiosh-mcp` Unit Suite (`test_mcp_package_tools`)
```
running 1 test
test tests::test_mcp_package_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.24s
```

### 4. Master Package Test Matrix (`tools/test_package_suites.py`)
Criteria `PM1..PM8` all pass.
