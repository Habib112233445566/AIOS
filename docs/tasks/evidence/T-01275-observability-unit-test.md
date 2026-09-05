# T-01275: Package Management / Observability - Unit Test

## Overview
Task `T-01275` establishes automated unit testing for the Package Management Observability Subsystem, implementing `code/aiosh-rust/aiosh-core/tests/test_package_observability.rs` with 6 dedicated test cases covering criteria `PO1..PO6`:

1. **`test_po1_inventory_completeness_and_empty_store`**:
   - Tests empty store boundary condition: zero counts, empty maps, zero sizes, zero policy compliance/violations.
   - Tests default store: verifies inventory completeness invariant where the sum of state breakdowns, format breakdowns, and architecture breakdowns all match `total_packages`.
2. **`test_po2_state_format_arch_breakdown_distributions`**:
   - Tests heterogeneous packages across multiple states (`installed`, `available`, `upgradable`, `pending_removal`), formats (`deb`, `apk`, `flatpak`), and architectures (`amd64`, `aarch64`, `x86_64`).
   - Verifies accurate counts in each breakdown map.
3. **`test_po3_footprint_and_capacity_telemetry`**:
   - Tests that `total_installed_size_bytes` strictly aggregates only installed and upgradable packages.
   - Verifies arithmetic mean computation for `average_package_size_bytes`.
4. **`test_po4_dependency_distribution_histogram`**:
   - Verifies dependency count categorization into the four canonical buckets: `"0"`, `"1-5"`, `"6-10"`, and `"11+"`.
5. **`test_po5_policy_compliance_and_prohibited_package_detection`**:
   - Tests mixed compliance: compliant package, prohibited package (`telnet`), and non-compliant package lacking mandatory checksum (`no-hash-pkg`).
   - Verifies `policy_compliant_count`, `policy_violations_count`, and detection of prohibited packages.
6. **`test_po6_serialization_and_negative_boundary_matrix`**:
   - Tests roundtrip JSON serialization and deserialization (`to_json_pretty()`).
   - Tests format and state conversion helper functions.

## Test Execution Results
`cargo test --manifest-path code/aiosh-rust/Cargo.toml --test test_package_observability`
```
running 6 tests
test test_po3_footprint_and_capacity_telemetry ... ok
test test_po2_state_format_arch_breakdown_distributions ... ok
test test_po4_dependency_distribution_histogram ... ok
test test_po1_inventory_completeness_and_empty_store ... ok
test test_po5_policy_compliance_and_prohibited_package_detection ... ok
test test_po6_serialization_and_negative_boundary_matrix ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
