# Task Evidence: T-01808 - Network Bootstrap / Data Model: Hardening

## Metadata
- **Task ID:** `T-01808`
- **Sub-Epic:** Sub-Epic 1: Network Bootstrap / Data Model
- **Component:** `code/aiosh-rust/aiosh-core/src/network.rs`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. Hardening Implemented

In response to the threat model formulated in `T-01807`, the following hardening controls were implemented in `aiosh-core::network`:

1. **Denial-of-Service Caps**:
   - `MAX_INTERFACES`: 1,024 interfaces per host state.
   - `MAX_ROUTES`: 4,096 routing table entries per host state.
   - `MAX_ADDRESSES_PER_IFACE`: 64 IP addresses per interface.
   - `MAX_FLAGS_PER_IFACE`: 32 status flags per interface.
   - `MAX_DNS_NAMESERVERS`: 32 resolver IP addresses per host state.
   - `MAX_DNS_SEARCH_DOMAINS`: 32 search domains per host state.

2. **Interface Level Hardening**:
   - Duplicate IP detection: Rejects multiple identical IP assignments to the same interface.
   - Flag validation: Length capped at 32 chars, control characters rejected.

3. **Hostname Validation (RFC 1123)**:
   - Length capped at 255 characters.
   - Characters restricted to ASCII alphanumeric, `.`, and `-`.
   - Rejects leading or trailing `.` or `-`.

4. **Deterministic Ordering (NET6)**:
   - `NetworkState::validate` verifies interfaces are sorted alphabetically by interface name.
   - `NetworkState::validate` verifies routes are sorted by metric ascending, then destination CIDR ascending.

5. **DNS Configuration Hardening**:
   - `DnsConfig::validate` validates every nameserver is a valid IPv4/IPv6 address and search domains are well-formed.

---

## 2. Test Verification

Added `test_network_hardening_caps` in `code/aiosh-rust/aiosh-core/tests/test_network.rs` verifying:
- Enforcement of `MAX_ADDRESSES_PER_IFACE` cap.
- Rejection of duplicate IP addresses on an interface.
- Enforcement of `MAX_FLAGS_PER_IFACE` cap.
- Enforcement of `MAX_DNS_NAMESERVERS` cap.
- Rejection of invalid hostnames (leading `-`, spaces, non-ASCII characters).
- Successful validation of well-formed network states.
