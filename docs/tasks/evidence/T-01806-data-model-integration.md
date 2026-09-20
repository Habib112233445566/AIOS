# Task Evidence: T-01806 - Network Bootstrap / Data Model: Integration

## Metadata
- **Task ID:** `T-01806`
- **Sub-Epic:** Sub-Epic 1: Network Bootstrap / Data Model
- **Component:** `code/aiosh-cli/tests/test_network_smoke.py`
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Integration Smoke Test Execution
Executed `pytest code/aiosh-cli/tests/test_network_smoke.py -v`:

```text
code/aiosh-cli/tests/test_network_smoke.py::test_net1_interface_name PASSED [ 16%]
code/aiosh-cli/tests/test_network_smoke.py::test_net2_mac_address PASSED [ 33%]
code/aiosh-cli/tests/test_network_smoke.py::test_net3_ip_address PASSED  [ 50%]
code/aiosh-cli/tests/test_network_smoke.py::test_net4_mtu PASSED         [ 66%]
code/aiosh-cli/tests/test_network_smoke.py::test_net5_routes PASSED      [ 83%]
code/aiosh-cli/tests/test_network_smoke.py::test_net6_network_state_schema_and_ordering PASSED [100%]

============================== 6 passed in 0.20s ==============================
```

## Invariants Verified (NET1..NET6)
- **NET1**: Interface name validation (non-empty, $\le 15$ characters matching Linux `IFNAMSIZ - 1`, alphanumeric plus `.`, `_`, `-`, rejecting traversal and illegal characters) verified.
- **NET2**: MAC address format validation (6 colon-delimited hex octets or empty/none) verified.
- **NET3**: IP address prefix bounds and family validation (IPv4 $\le 32$, IPv6 $\le 128$) verified.
- **NET4**: MTU bounded range validation ($[68, 65535]$) verified.
- **NET5**: Route validity (non-empty destination CIDR, non-negative metric, presence of gateway or interface) verified.
- **NET6**: Deterministic ordering (interfaces alphabetically by name, routes by metric then destination) and JSON schema roundtrip verified.
