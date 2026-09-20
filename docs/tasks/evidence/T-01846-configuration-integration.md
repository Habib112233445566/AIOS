# Task Evidence: T-01846 - Network Bootstrap / configuration: Integration

## Summary
Implemented and executed the cross-surface integration smoke test suite for the Network Bootstrap Configuration subsystem in `code/aiosh-cli/tests/test_network_config_smoke.py`.

## Verification Details
1. **Scope & Invariants**:
   - `NCONF1`: Path hygiene verification (null bytes, control characters, string length limits $\le 1024$, and parent directory traversal `..` prevention).
   - `NCONF2`: Boundary checks on interface, route, and DNS server capacities.
   - `NCONF3`: Verification of maximum serialized payload bounds and timeout bounds.
   - `NCONF4`: Validation of fallback DNS server address parsing (IPv4 and IPv6) and cap enforcement.
   - `NCONF5`: Ingestion of environment variables (`AIOS_NETWORK_*`).
   - `NCONF6`: Lossless JSON persistence, roundtrip deserialization, and enforcement of the 1 MB configuration file size limit.
2. **Execution Results**:
   - Command: `python code/aiosh-cli/tests/test_network_config_smoke.py`
   - Output:
     ```
     Running Network Bootstrap Configuration Smoke Tests (T-01846)...
     PASS: test_nconf1_path_hygiene
     PASS: test_nconf2_capacity_limits
     PASS: test_nconf3_resource_bounds
     PASS: test_nconf4_fallback_dns
     PASS: test_nconf5_environment_ingestion
     PASS: test_nconf6_persistence
     ALL NETWORK CONFIG SMOKE TESTS PASSED.
     ```
