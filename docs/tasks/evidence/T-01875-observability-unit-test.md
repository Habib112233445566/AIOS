# Task Evidence: T-01875 - Network Bootstrap / observability: Unit Test

## 1. Overview
- **Task ID**: `T-01875`
- **Sub-Epic**: 8 (Network Bootstrap Observability)
- **Goal**: Author and execute comprehensive unit tests for Network Bootstrap Observability in `code/aiosh-rust/aiosh-core/tests/test_network_observability.rs`.

---

## 2. Test Cases & Coverage
1. `test_nobs1_procfs_parsing`: Validates non-blocking parsing of `/proc/net/dev` lines, token extraction, and correct mapping to rx/tx counter fields.
2. `test_nobs2_missing_procfs_fallback`: Validates graceful fallback to empty statistics when `/proc/net/dev` is absent.
3. `test_nobs2_sysfs_carrier_enrichment`: Validates discovery and parsing of sysfs `carrier` attributes (`1` -> `Some(true)`, `0` -> `Some(false)`).
4. `test_nobs3_health_healthy`: Validates `Healthy` diagnosis for connected host with default gateway and DNS.
5. `test_nobs3_health_degraded_no_default_route`: Validates `Degraded` diagnosis when default route is missing.
6. `test_nobs3_health_degraded_no_dns`: Validates `Degraded` diagnosis when DNS resolvers are missing.
7. `test_nobs3_health_degraded_high_drops`: Validates `Degraded` diagnosis when packet drop rate exceeds 5%.
8. `test_nobs3_health_critical_all_interfaces_down`: Validates `Critical` diagnosis when all external interfaces are down.
9. `test_nobs3_health_critical_no_route_and_no_dns`: Validates `Critical` diagnosis when both default route and DNS resolvers are missing.
10. `test_nobs4_history_ring_buffer_eviction`: Validates bounded history ring buffer behavior and FIFO eviction of oldest snapshots.
11. `test_nobs6_persistence_atomic_and_path_hygiene`: Validates JSON serialization, roundtrip deserialization, atomic write, and path hygiene checks (`..` and control characters rejected).
12. `test_nobs6_oversized_snapshot_rejected`: Validates rejection of oversized snapshots exceeding `MAX_OBSERVABILITY_FILE_BYTES` (1 MB).

---

## 3. Test Execution Verification
Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_network_observability`

```text
running 12 tests
test test_nobs2_missing_procfs_fallback ... ok
test test_nobs3_health_critical_all_interfaces_down ... ok
test test_nobs2_sysfs_carrier_enrichment ... ok
test test_nobs3_health_critical_no_route_and_no_dns ... ok
test test_nobs3_health_degraded_high_drops ... ok
test test_nobs1_procfs_parsing ... ok
test test_nobs3_health_degraded_no_default_route ... ok
test test_nobs3_health_degraded_no_dns ... ok
test test_nobs3_health_healthy ... ok
test test_nobs4_history_ring_buffer_eviction ... ok
test test_nobs6_oversized_snapshot_rejected ... ok
test test_nobs6_persistence_atomic_and_path_hygiene ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```
Status: PASS (12/12 passed, 0 failures, 0 warnings).
