# Task Evidence: T-01818 - Network Bootstrap / Core Service: Hardening

## Metadata
- **Task ID:** `T-01818`
- **Sub-Epic:** Sub-Epic 2: Network Bootstrap / Core Service
- **Component:** `code/aiosh-rust/aiosh-core/src/network_service.rs`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. Hardening Implemented

In response to the threat vectors identified in `T-01817`, the following hardening controls were implemented in `NetworkService`:

1. **Bounded File Reading (`read_bounded_string`)**:
   - Replaced unbounded `fs::read_to_string` with `read_bounded_string` using `std::io::Read::take(max_bytes)`.
   - Prevents memory exhaustion or infinite read hangs from hostile fifos or oversized files (`NSERV6`).
2. **Resource Cap Constants**:
   - `MAX_SYSFS_FILE_BYTES = 64 * 1024` (64 KB for sysfs attribute files: `operstate`, `address`, `mtu`, `type`, `flags`).
   - `MAX_ROUTE_FILE_BYTES = 1024 * 1024` (1 MB for `/proc/net/route`).
   - `MAX_RESOLV_FILE_BYTES = 64 * 1024` (64 KB for `/etc/resolv.conf`).
3. **Interface Name Validation on Link Mutations**:
   - Enforced `validate_interface_name` on `bring_up` and `bring_down` (`NET1`), rejecting command injection or path traversal attempts.

---

## 2. Test Verification

Added `test_nserv_hardening_file_bounds` in `code/aiosh-rust/aiosh-core/tests/test_network_service.rs` verifying:
- Files exceeding `MAX_SYSFS_FILE_BYTES` (e.g. 100 KB payload) are strictly bounded and truncated to 64 KB without crashing or allocating excessive heap memory.
