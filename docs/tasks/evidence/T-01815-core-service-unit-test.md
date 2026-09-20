# Task Evidence: T-01815 - Network Bootstrap / Core Service: Unit Test

## Metadata
- **Task ID:** `T-01815`
- **Sub-Epic:** Sub-Epic 2: Network Bootstrap / Core Service
- **Component:** `code/aiosh-rust/aiosh-core/tests/test_network_service.rs`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. Unit Test Suite: `test_network_service.rs`

Implemented 6 comprehensive unit tests validating invariants `NSERV1..NSERV6`:

1. **`test_nserv1_mock_service_initialization`**:
   - Verifies `NetworkService::with_paths(...)` path configuration accessors for sysfs, procfs, and resolv.conf roots (`NSERV1`).
2. **`test_nserv2_scan_interfaces_and_fallback`**:
   - Tests mock sysfs scanning for complete (`eth0`), loopback (`lo`), and empty/partial directories (`wlan0`).
   - Confirms graceful degradation to defaults (`operstate=Unknown`, `mtu=1500`, `mac=None`) (`NSERV2`).
   - Confirms deterministic alphabetical sorting (`eth0`, `lo`, `wlan0`) (`NET6`).
   - Verifies `get_interface` and rejection of traversal/injection characters (`NET1`, `NSERV5`).
3. **`test_nserv3_route_parsing`**:
   - Verifies `/proc/net/route` little-endian hex parsing (`NSERV3`).
   - Tests default route extraction (`0.0.0.0/0` via gateway).
   - Tests subnet route extraction (`192.168.1.0/24`, `10.0.0.0/8`).
   - Verifies deterministic sorting by metric ascending (`NET6`).
4. **`test_nserv4_dns_parsing`**:
   - Verifies `/etc/resolv.conf` comment stripping, nameserver extraction, search domain extraction, and validation (`NSERV4`).
5. **`test_nserv5_bring_up_and_bring_down`**:
   - Verifies state transitions (`up` and `down`) in mock sysfs operstate files (`NSERV5`).
   - Verifies name validation rejects command injection and directory traversal.
6. **`test_nserv6_get_network_state_unified`**:
   - Verifies unified snapshot generation, combining interfaces, routes, and DNS, and passing `validate_invariants()` (`NET1..NET6`).
