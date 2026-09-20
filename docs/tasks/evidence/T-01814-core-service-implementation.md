# Task Evidence: T-01814 - Network Bootstrap / Core Service: Implementation

## Metadata
- **Task ID:** `T-01814`
- **Sub-Epic:** Sub-Epic 2: Network Bootstrap / Core Service
- **Component:** `code/aiosh-rust/aiosh-core/src/network_service.rs`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. Implementation Details

Implemented the full `NetworkService` logic in `code/aiosh-rust/aiosh-core/src/network_service.rs`:

1. **Path-Configurable Core Service**:
   - `NetworkService::with_paths(sysfs, procfs, resolv)` enables complete test isolation and mock execution without needing root or live Linux kernel filesystems (`NSERV1`).
2. **Interface Discovery (`scan_interfaces`, `get_interface`)**:
   - Scans sysfs directory entries.
   - Validates interface names using `validate_interface_name` (`NET1`), preventing directory traversal (`NSERV5`).
   - Gracefully parses `operstate`, `address`, `mtu`, `type`, and `flags` with safe defaults (`NSERV2`).
   - Decodes Linux interface flags (`0x1003` -> `UP`, `BROADCAST`, `LOOPBACK`, `RUNNING`, `MULTICAST`).
   - Deterministically sorts interfaces by name (`NET6`).
3. **Routing Table Inspection (`scan_routes`)**:
   - Parses `/proc/net/route` IPv4 table.
   - Decodes 8-character little-endian hexadecimal IP addresses and masks into CIDR format (`NSERV3`).
   - Converts mask to prefix length using `.count_ones()`.
   - Identifies default gateway routes (`0.0.0.0/0`).
   - Deterministically sorts routes by metric ascending, then destination ascending (`NET6`).
4. **DNS Configuration (`get_dns_config`)**:
   - Parses `/etc/resolv.conf`, strips comments, extracts `nameserver` and `search` lines (`NSERV4`).
   - Enforces caps on nameservers (32) and search domains (32).
5. **Unified Network Snapshot (`get_network_state`)**:
   - Combines interfaces, routes, and DNS into a unified snapshot.
   - Validates all invariants via `state.validate_invariants()` (`NET1..NET6`).
6. **Interface State Mutations (`bring_up`, `bring_down`)**:
   - Validates interface name against command injection / traversal (`NET1`).
   - Updates `operstate` when mock path exists (`NSERV5`).
