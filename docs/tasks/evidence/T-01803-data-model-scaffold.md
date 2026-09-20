# Task Evidence: T-01803 - Network Bootstrap / Data Model: Scaffold

## Metadata
- **Task ID:** `T-01803`
- **Sub-Epic:** Sub-Epic 1: Network Bootstrap / Data Model
- **Epic:** Phase 1 — Linux Base System & Bootable Target / Network Bootstrap
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Scaffolding Summary
1. Created `code/aiosh-rust/aiosh-core/src/network.rs`:
   - Data types: `InterfaceType`, `OperState`, `IpFamily`, `IpAddress`, `Route`, `DnsConfig`, `NetworkInterface`, `NetworkState`.
   - Invariant validation functions: `validate_interface_name`, `validate_mac_address`, `validate_ip_address`, `validate_mtu`, `validate_route`, `validate_network_interface`, `validate_network_state`.
   - Bounds: `MAX_IFACE_NAME_LEN = 15`, `MIN_MTU = 68`, `MAX_MTU = 65535`, `MAX_INTERFACES = 1024`, `MAX_ROUTES = 4096`.
   - Deterministic sorting of interfaces and routes on `NetworkState`.
2. Exported `pub mod network;` and re-exports in `code/aiosh-rust/aiosh-core/src/lib.rs`.
