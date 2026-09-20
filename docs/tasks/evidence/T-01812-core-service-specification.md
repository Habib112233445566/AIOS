# Task Evidence: T-01812 - Network Bootstrap / Core Service: Specification

## Metadata
- **Task ID:** `T-01812`
- **Sub-Epic:** Sub-Epic 2: Network Bootstrap / Core Service
- **Component:** `code/aiosh-rust/aiosh-core/src/network_service.rs`
- **Date:** 2026-09-20
- **Status:** COMPLETED

---

## 1. Component Specification: `NetworkService`

### Struct Definition

```rust
pub struct NetworkService {
    sysfs_net_root: PathBuf,
    procfs_root: PathBuf,
    resolv_conf_path: PathBuf,
}
```

### Public API Specification

1. **Constructors**:
   - `NetworkService::new() -> Self`: Defaults to `/sys/class/net`, `/proc/net`, and `/etc/resolv.conf`.
   - `NetworkService::with_paths(sysfs: impl Into<PathBuf>, procfs: impl Into<PathBuf>, resolv: impl Into<PathBuf>) -> Self`: Allows custom mock paths for hermetic cross-platform execution.
2. **Interface Discovery**:
   - `fn scan_interfaces(&self) -> Result<Vec<NetworkInterface>, String>`:
     - Scans `sysfs_net_root`.
     - Validates interface names with `validate_interface_name` (`NET1`).
     - Reads `operstate`, `address`, `mtu`, `type`, `flags`.
     - Deterministically sorts alphabetically by interface name (`NET6`).
   - `fn get_interface(&self, name: &str) -> Result<Option<NetworkInterface>, String>`:
     - Validates name format before filesystem lookup.
3. **Routing Table Inspection**:
   - `fn scan_routes(&self) -> Result<Vec<Route>, String>`:
     - Parses `procfs_root/route`.
     - Decodes 8-character little-endian hexadecimal IPv4 addresses (`NSERV3`).
     - Converts hex netmask to CIDR prefix length.
     - Deterministically sorts by metric ascending, then destination ascending (`NET6`).
4. **DNS Resolver Inspection**:
   - `fn get_dns_config(&self) -> Result<DnsConfig, String>`:
     - Parses `resolv_conf_path`.
     - Strips comments (`#`, `;`), extracts `nameserver` and `search` lines.
     - Validates nameservers and search domains (`NSERV4`).
5. **Full Network State Snapshot**:
   - `fn get_network_state(&self) -> Result<NetworkState, String>`:
     - Combines interfaces, routes, and DNS into a unified snapshot.
     - Validates state invariants (`NET1..NET6`).
6. **Interface State Mutations**:
   - `fn bring_up(&self, iface: &str) -> Result<(), String>`:
     - Validates name (`NET1`). Sets operstate to `up` or invokes link up.
   - `fn bring_down(&self, iface: &str) -> Result<(), String>`:
     - Validates name (`NET1`). Sets operstate to `down` or invokes link down.

---

## 2. Invariants & Error Handling

- **`NSERV1`**: Offline mockability via `with_paths`.
- **`NSERV2`**: Missing sysfs files fall back to defaults (`operstate=Unknown`, `mtu=1500`, `mac=None`).
- **`NSERV3`**: Hex route parsing checks for exact 8-hex characters and handles endianness safely.
- **`NSERV4`**: Invalid DNS entries are ignored or error on state validation.
- **`NSERV5`**: Name validation prevents path traversal in `sysfs_net_root.join(name)`.
- **`NSERV6`**: Maximum limits enforced: interfaces $\le 1024$, routes $\le 4096$, nameservers $\le 32$.
