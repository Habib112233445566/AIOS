# Task Evidence: T-01801 - Network Bootstrap / Data Model: Research

## Metadata
- **Task ID:** `T-01801`
- **Sub-Epic:** Sub-Epic 1: Network Bootstrap / Data Model
- **Epic:** Phase 1 — Linux Base System & Bootable Target / Network Bootstrap (`T-01801` through `T-01900`)
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Research Scope & Objectives
Establish facts, constraints, and architecture for the Network Bootstrap data model:
1. Examine Linux kernel networking abstractions (`/sys/class/net`, netlink, `iproute2`).
2. Identify core data structures: network interfaces, link states, IP addressing (IPv4/IPv6, CIDR, DHCP/static), routes, and DNS configurations.
3. Formulate data model invariants `NET1..NET6`.
4. Separate facts from assumptions.

## Facts vs. Assumptions

### Facts (Authoritative Linux Subsystem Constraints)
1. **Interface Naming (`IFNAMSIZ`)**:
   - In Linux (`<linux/if.h>`), network interface names have a maximum length of 16 bytes including null terminator (`IFNAMSIZ = 16`), meaning maximum string length is 15 characters.
   - Allowed characters in interface names are alphanumeric characters, dots, hyphens, and underscores (`^[a-zA-Z0-9_.-]+$`).
2. **Link Operational States**:
   - `/sys/class/net/<iface>/operstate` reports states defined by RFC 2863: `up`, `down`, `dormant`, `testing`, `lowerlayerdown`, `notpresent`, `unknown`.
3. **MAC Address Standards**:
   - Ethernet MAC addresses are 48-bit (6 octets), conventionally represented as 12 hexadecimal digits separated by colons (`XX:XX:XX:XX:XX:XX`).
4. **MTU Limits**:
   - Minimum IPv4 MTU is 68 bytes (RFC 791).
   - Minimum IPv6 MTU is 1280 bytes (RFC 8200).
   - Standard Ethernet MTU is 1500 bytes. Jumbo frames range up to 9000 bytes. Absolute theoretical maximum is 65535 bytes.
5. **IP & Routing**:
   - IPv4 prefix length: 0 to 32.
   - IPv6 prefix length: 0 to 128.
   - Default route is `0.0.0.0/0` (IPv4) or `::/0` (IPv6).

### Assumptions
1. AIOS network bootstrap data model will provide an in-memory representation capable of being populated from `/sys/class/net` and `/proc/net` or serialized to/from declarative network configuration files (systemd-networkd, Netplan, or custom JSON manifests).
2. The data model should be independent of specific network daemons (NetworkManager, systemd-networkd, wicked) so that it can serve as a universal state representation.

## Invariants Formulated (NET1..NET6)
- **NET1 (Interface Identification & Charset)**: Interface names must be 1 to 15 characters long, matching ASCII alphanumeric, dot, hyphen, or underscore.
- **NET2 (MAC Address Hygiene)**: MAC addresses must match valid 6-octet colon-separated hex format or be None/empty (e.g. for loopback or point-to-point links).
- **NET3 (IP Address & CIDR Hygiene)**: IP addresses must be parseable as standard IPv4 or IPv6, with prefix lengths within [0, 32] or [0, 128].
- **NET4 (MTU Bounds)**: MTU must be bounded in [68, 65535].
- **NET5 (Route Integrity)**: Routes must specify a valid destination prefix, a positive metric, and a valid gateway or interface target.
- **NET6 (Deterministic Canonical Serialization)**: Collections of interfaces, addresses, and routes must be deterministically ordered (by interface name, IP address, or metric).
