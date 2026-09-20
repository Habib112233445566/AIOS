# Task Evidence: T-01895 - Network Bootstrap / recovery & validation: Unit Test

## 1. Overview
- **Task ID**: `T-01895`
- **Sub-Epic**: 10 (Network Bootstrap Recovery & Validation)
- **Goal**: Author and execute comprehensive unit tests for Network Recovery & Validation in `test_network_recovery.rs`.

---

## 2. Test Cases & Invariant Verification
1. `test_nval1_interface_counts_parity`: Verifies `valid_interfaces + invalid_interfaces == total_interfaces` (`NVAL1`).
2. `test_nval2_dangling_routes_detection_and_pruning`: Verifies detection of routes pointing to non-existent interfaces and their automatic pruning (`NVAL2`).
3. `test_nval3_dns_missing_and_fallback`: Verifies unconfigured DNS is diagnosed as unhealthy and healed by injecting fallback resolvers (`1.1.1.1`, `8.8.8.8`) (`NVAL3`).
4. `test_nval4_loopback_restoration`: Verifies absence of loopback interface is diagnosed and automatically restored with standard IPv4 and IPv6 loopback addresses (`NVAL4`).
5. `test_nval4_healthy_state_passes_clean`: Verifies an already-healthy network state executes without mutations and reports `NoneRequired` (`NVAL4`).
6. `test_nval5_quarantine_corrupted_file`: Verifies corrupted/malformed JSON file on disk is quarantined to `<path>.bak.<timestamp>` preserving byte-for-byte contents, and original file is restored to valid configuration (`NVAL5`).
7. `test_nval6_path_hygiene_and_validation`: Verifies path rejection for directory traversal (`..`), control characters, and non-`.json` extensions (`NVAL6`).
8. `test_nval6_oversized_store_rejected`: Verifies rejection of files exceeding `MAX_NETWORK_STORE_SIZE` (1 MB) (`NVAL6`).

---

## 3. Test Execution Verification
Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_network_recovery`

```text
running 8 tests
test test_nval1_interface_counts_parity ... ok
test test_nval2_dangling_routes_detection_and_pruning ... ok
test test_nval3_dns_missing_and_fallback ... ok
test test_nval4_healthy_state_passes_clean ... ok
test test_nval4_loopback_restoration ... ok
test test_nval6_path_hygiene_and_validation ... ok
test test_nval6_oversized_store_rejected ... ok
test test_nval5_quarantine_corrupted_file ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
```
Status: Verified & Passed (8/8 tests passing, 0 warnings).
