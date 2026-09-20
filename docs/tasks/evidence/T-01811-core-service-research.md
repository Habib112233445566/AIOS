# Task Evidence: T-01811 - Network Bootstrap / Core Service: Research

## Metadata
- **Task ID:** `T-01811`
- **Sub-Epic:** Sub-Epic 2: Network Bootstrap / Core Service
- **Component:** `code/aiosh-rust/aiosh-core/src/network_service.rs`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. Research Overview

Sub-Epic 2 introduces `NetworkService`, the core component responsible for discovering, scanning, querying, and managing network interfaces, routing tables, and DNS configuration in AIOS.

### Linux Kernel Interfaces Researched

1. **Sysfs Network Class (`/sys/class/net/`)**:
   - Directory hierarchy where each subdirectory represents a network interface.
   - Attributes read per interface:
     - `operstate`: ASCII string (`up`, `down`, `unknown`, `dormant`, `lowerlayerdown`).
     - `address`: MAC address in colon-separated hex format.
     - `mtu`: Decimal string representation of MTU.
     - `flags`: Hexadecimal bitmask of interface flags (`IFF_UP=0x1`, `IFF_BROADCAST=0x2`, `IFF_LOOPBACK=0x8`, `IFF_RUNNING=0x40`, `IFF_MULTICAST=0x1000`).
     - `type`: ARPHRD hardware type (1 for Ethernet, 772 for Loopback).
2. **Procfs IPv4 Routing (`/proc/net/route`)**:
   - Tab/whitespace-delimited table with columns: `Iface`, `Destination`, `Gateway`, `Flags`, `RefCnt`, `Use`, `Metric`, `Mask`, `MTU`, `Window`, `IRTT`.
   - `Destination` and `Gateway` are stored in 8-character little-endian hexadecimal format (e.g., `0101A8C0` represents `192.168.1.1`).
   - `Mask` is also 8-character hex, converted to prefix length (e.g., `00FFFFFF` is `/24`).
3. **DNS Resolver Configuration (`/etc/resolv.conf`)**:
   - Parses `nameserver <ip>` and `search <domain...>` directives.
   - Ignores comments prefixed with `#` or `;`.

---

## 2. Invariants Formulated (`NSERV1..NSERV6`)

- **`NSERV1` (Hermetic Mockability)**: `NetworkService` must accept configurable root paths (`sysfs_net_root`, `procfs_root`, `resolv_conf_path`) allowing offline, hermetic testing on non-Linux hosts.
- **`NSERV2` (Graceful Sysfs Degradation)**: Missing or unreadable sysfs files must fall back to safe defaults (`operstate=Unknown`, `mtu=1500`, `mac=None`) rather than aborting the scan.
- **`NSERV3` (Hex Route Decoding Safety)**: Little-endian hex IPv4 address decoding must be bounds-checked and validated without panics.
- **`NSERV4` (DNS Resolver Sanitization)**: Nameservers must be validated against IP parsing; search domains must be validated against RFC 1123.
- **`NSERV5` (Interface Link Operations)**: Interface state modification methods (`bring_up`, `bring_down`) must validate interface names (`NET1`) and return structured error codes.
- **`NSERV6` (Bounded Resource Limits)**: All directory iterations and file parsers must respect hard caps: `MAX_INTERFACES` (1024), `MAX_ROUTES` (4096), `MAX_DNS_NAMESERVERS` (32), and maximum file size (1 MB).
