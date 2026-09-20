# Task Evidence: T-01816 - Network Bootstrap / Core Service: Integration

## Metadata
- **Task ID:** `T-01816`
- **Sub-Epic:** Sub-Epic 2: Network Bootstrap / Core Service
- **Component:** `code/aiosh-cli/tests/test_network_service_smoke.py`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. Integration Smoke Test Suite Execution

Executed `pytest code/aiosh-cli/tests/test_network_service_smoke.py -v`:

```text
code/aiosh-cli/tests/test_network_service_smoke.py::test_nserv1_hermetic_mock_paths PASSED [ 16%]
code/aiosh-cli/tests/test_network_service_smoke.py::test_nserv2_sysfs_interface_scanning_and_fallback PASSED [ 33%]
code/aiosh-cli/tests/test_network_service_smoke.py::test_nserv3_proc_net_route_parsing PASSED [ 50%]
code/aiosh-cli/tests/test_network_service_smoke.py::test_nserv4_resolv_conf_parsing PASSED [ 66%]
code/aiosh-cli/tests/test_network_service_smoke.py::test_nserv5_interface_link_mutation_safety PASSED [ 83%]
code/aiosh-cli/tests/test_network_service_smoke.py::test_nserv6_state_schema_and_ordering PASSED [100%]

============================== 6 passed in 0.26s ==============================
```

---

## 2. Invariants Verified (`NSERV1..NSERV6`)

- **`NSERV1`**: Hermetic mockability with custom temporary directories confirmed.
- **`NSERV2`**: Graceful fallback on missing sysfs attributes (`operstate=Unknown`, `mtu=1500`, `mac=None`) confirmed.
- **`NSERV3`**: `/proc/net/route` little-endian hex parsing (`0101A8C0` to `192.168.1.1`) and netmask-to-CIDR calculation confirmed.
- **`NSERV4`**: `/etc/resolv.conf` comment stripping, nameserver and search domain parsing confirmed.
- **`NSERV5`**: Link state mutations (`bring_up`, `bring_down`) and interface name validation against directory traversal confirmed.
- **`NSERV6`**: Deterministic state sorting and JSON schema parity confirmed.
