# Task Evidence: T-01805 - Network Bootstrap / Data Model: Unit Test

## Metadata
- **Task ID:** `T-01805`
- **Sub-Epic:** Sub-Epic 1: Network Bootstrap / Data Model
- **Epic:** Phase 1 — Linux Base System & Bootable Target / Network Bootstrap
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Unit Test Suite Coverage
1. `test_net1_interface_name_validation`:
   - Validates legal interface names (`eth0`, `lo`, `enp3s0`, `br-lan`, `vlan.100`).
   - Asserts rejection of empty names, names exceeding 15 characters, spaces, semicolons, and control characters (`NET1`).
2. `test_net2_mac_address_validation`:
   - Asserts valid 6-octet colon-separated hex MAC formatting (`NET2`).
   - Rejects non-hex characters and invalid octet counts.
3. `test_net3_ip_address_and_cidr_validation`:
   - Validates IPv4 and IPv6 address parsing, CIDR notation (`10.0.0.1/8`, `2001:db8::1/64`), and prefix boundaries ($\le 32$ for IPv4, $\le 128$ for IPv6) (`NET3`).
4. `test_net4_mtu_bounds`:
   - Verifies MTU bounds checking within $[68, 65535]$ (`NET4`).
5. `test_net5_route_validation`:
   - Tests default and subnet routing rules, gateway/interface presence, and CIDR validation (`NET5`).
6. `test_net6_network_state_deterministic_ordering_and_queries`:
   - Validates deterministic alphabetical interface ordering and metric-based route ordering (`NET6`).
   - Verifies state query helpers: `interfaces_up()`, `default_gateway()`, `default_gateway_v6()`, `find_interface_by_ip()`.
   - Tests JSON serialization and deserialization roundtrip.
