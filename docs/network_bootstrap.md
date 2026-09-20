# AIOS Network Bootstrap Subsystem

## 1. Overview & Architecture

The Network Bootstrap subsystem provides safe, deterministic, and strongly-typed network interface discovery, IP address configuration, routing table management, and DNS resolution for AIOS. It interfaces with the underlying Linux kernel networking abstractions via sysfs (`/sys/class/net/`), procfs (`/proc/net/route`), and resolvconf (`/etc/resolv.conf`).

The subsystem guarantees that network state manipulation is bounded, deterministic, and free of command injection, path traversal, or resource exhaustion vulnerabilities.

---

## 2. Network Data Model

Defined in `code/aiosh-rust/aiosh-core/src/network.rs`.

### Core Data Structures

#### `InterfaceType`
Functional classification of network interfaces:
- `Loopback` (`lo`)
- `Ethernet` (`eth*`, `en*`, `eno*`, `ens*`, `enp*`)
- `Wireless` (`wlan*`, `wl*`, `wlp*`)
- `Bridge` (`br*`)
- `Bond` (`bond*`)
- `Vlan` (`vlan*`, `*.100`)
- `TunTap` (`tun*`, `tap*`)
- `Other`

#### `OperState`
Operational link state (RFC 2863):
- `Up`: Interface is up and operational.
- `Down`: Interface is administratively or operationally down.
- `Dormant`: Interface is waiting for an external event (e.g. 802.1X).
- `LowerLayerDown`: Lower physical or virtual layer is down.
- `Unknown`: State cannot be determined.

#### `IpAddress`
Represents an IPv4 or IPv6 address with CIDR prefix length:
- `address`: String representation of the IP address (e.g., `"192.168.1.100"` or `"fe80::1"`).
- `prefix_len`: Prefix length (`0..=32` for IPv4, `0..=128` for IPv6).
- `family`: `IpFamily::V4` or `IpFamily::V6`.
- Helper: `IpAddress::from_cidr("192.168.1.1/24")`.

#### `Route`
Routing table entry:
- `destination`: Destination CIDR (e.g., `"0.0.0.0/0"`, `"192.168.1.0/24"`, `"::/0"`).
- `gateway`: Optional next-hop gateway IP (e.g., `"192.168.1.1"`).
- `interface`: Optional outbound interface name (e.g., `"eth0"`).
- `metric`: Integer route metric (lower values indicate higher priority).

#### `DnsConfig`
DNS resolver configuration:
- `nameservers`: Ordered list of resolver IP addresses (max 32).
- `search_domains`: Ordered list of search domain suffixes (max 32).

#### `NetworkInterface`
Individual interface representation:
- `name`: Interface name ($\le 15$ characters matching `^[a-zA-Z0-9_.-]+$`).
- `iftype`: `InterfaceType`.
- `operstate`: `OperState`.
- `mac_address`: Optional 6-octet colon-delimited MAC address.
- `mtu`: Maximum Transmission Unit ($68 \le \text{MTU} \le 65535$).
- `ip_addresses`: List of assigned IP addresses (max 64 per interface).
- `flags`: Interface status flags (e.g., `UP`, `BROADCAST`, `RUNNING`, `MULTICAST`).

#### `NetworkState`
Complete host network state snapshot:
- `timestamp`: ISO 8601 UTC timestamp of snapshot creation.
- `hostname`: RFC 1123 compliant hostname ($\le 255$ characters).
- `interfaces`: Deterministically sorted list of network interfaces (sorted alphabetically by name).
- `routes`: Deterministically sorted routing table (sorted by metric ascending, then destination ascending).
- `dns`: `DnsConfig`.

---

## 3. Invariants & Safety Guarantees

| Invariant | Name | Description & Enforcement |
|:---|:---|:---|
| **`NET1`** | Interface Name Safety | Name must be non-empty, $\le 15$ characters (Linux `IFNAMSIZ - 1`), matching `^[a-zA-Z0-9_.-]+$`. Rejects control characters, path separators (`/`, `\`), traversal (`..`), and shell metacharacters. |
| **`NET2`** | MAC Address Format | MAC address must be 6 colon-delimited hex octets (`^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$`) or empty/None for interfaces without MACs. |
| **`NET3`** | IP Prefix & Family | Prefix length must be bounded: $\le 32$ for IPv4, $\le 128$ for IPv6. Family specification must match IP string address family. |
| **`NET4`** | MTU Bounded Range | MTU must satisfy $68 \le \text{MTU} \le 65535$. Rejects zero, sub-minimum ($< 68$), and overflow values. |
| **`NET5`** | Route Validity | Route destination must be a valid CIDR, metric must be non-negative, and at least one of gateway or interface must be present. |
| **`NET6`** | Deterministic Ordering | Interfaces are sorted alphabetically by name; routes are sorted by metric ascending, then destination CIDR ascending. |

---

## 4. Hardening Limits & Resource Caps

To protect against denial-of-service and memory exhaustion attacks:
- `MAX_INTERFACES`: 1,024 interfaces per host state.
- `MAX_ROUTES`: 4,096 routing table entries per host state.
- `MAX_ADDRESSES_PER_IFACE`: 64 IP addresses per interface.
- `MAX_FLAGS_PER_IFACE`: 32 status flags per interface.
- `MAX_DNS_NAMESERVERS`: 32 resolver IP addresses per host state.
- `MAX_DNS_SEARCH_DOMAINS`: 32 search domains per host state.
- `MAX_IFACE_NAME_LEN`: 15 characters.

---

## 5. Network Core Service Architecture & Discovery (`NetworkService`)

Defined in `code/aiosh-rust/aiosh-core/src/network_service.rs`.

The `NetworkService` handles the active discovery of physical and virtual network interfaces, system routing tables, and DNS configuration directly from the Linux kernel and operating system.

### Core Capabilities

1. **Hermetic Mockability (`NSERV1`)**:
   - `NetworkService::with_paths(sysfs_net_root, procfs_root, resolv_conf_path)` allows callers to redirect all discovery and mutation calls to mock filesystem hierarchies.
   - Enables 100% offline, cross-platform unit and integration testing without requiring a live Linux kernel or root privileges.
2. **Interface Discovery & Graceful Degradation (`NSERV2`)**:
   - Iterates through `/sys/class/net/*` (or configured mock path).
   - Validates interface directory names against `NET1` to prevent path traversal or special character injection (`NSERV5`).
   - Gracefully falls back to defaults when optional sysfs files are missing (`operstate=Unknown`, `mtu=1500`, `mac=None`).
   - Decodes Linux interface flags (`IFF_UP`, `IFF_BROADCAST`, `IFF_LOOPBACK`, `IFF_RUNNING`, `IFF_MULTICAST`).
   - Deterministically sorts discovered interfaces alphabetically by name (`NET6`).
3. **Routing Table Parsing (`NSERV3`)**:
   - Reads and parses `/proc/net/route`.
   - Decodes little-endian hexadecimal IPv4 destinations, gateways, and netmasks.
   - Calculates CIDR prefix length using `.count_ones()` on decoded netmasks.
   - Identifies default gateway routes (`0.0.0.0/0`).
   - Deterministically sorts routes by metric ascending, then destination ascending (`NET6`).
4. **DNS Configuration Inspection (`NSERV4`)**:
   - Parses `/etc/resolv.conf`, safely stripping comment lines (`#`, `;`).
   - Validates and enforces caps on nameservers (`MAX_DNS_NAMESERVERS = 32`) and search domains (`MAX_DNS_SEARCH_DOMAINS = 32`).
5. **Interface Link Mutations (`bring_up`, `bring_down`) (`NSERV5`)**:
   - Validates interface names before mutation to prevent command injection or directory traversal.
   - Updates mock operstate files in testing mode.
6. **Bounded Resource Limits (`NSERV6`)**:
   - All file reads are strictly bounded via `read_bounded_string`:
     - `MAX_SYSFS_FILE_BYTES = 64 KB`
     - `MAX_ROUTE_FILE_BYTES = 1 MB`
     - `MAX_RESOLV_FILE_BYTES = 64 KB`
   - Bounded collections: `MAX_INTERFACES` (1024), `MAX_ROUTES` (4096), `MAX_DNS_NAMESERVERS` (32).
