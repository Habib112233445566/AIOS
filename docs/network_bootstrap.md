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

---

## 6. Network Bootstrap CLI Surface (`aiosh net` / `aiosh network`)

The CLI surface provides operators and agents with terminal and machine-readable inspection and control of network interfaces, routing tables, and DNS configuration.

### Subcommands

| Subcommand | Arguments | Description | Exit Codes |
|------------|-----------|-------------|------------|
| `list` | None | Lists all discovered interfaces in table or JSON format | 0, 1, 2 |
| `show` | `<interface>` | Shows detailed interface attributes (type, operstate, MAC, MTU, flags) | 0, 1, 2 |
| `routes` | None | Displays IPv4 routing table with destinations, gateways, and metrics | 0, 1, 2 |
| `dns` | None | Displays configured DNS nameservers and search domains | 0, 1, 2 |
| `state` | None | Generates a complete host network state snapshot | 0, 1, 2 |
| `up` | `<interface>` | Brings the specified interface link up | 0, 1, 2 |
| `down` | `<interface>` | Brings the specified interface link down | 0, 1, 2 |

### CLI Options

- `--sysfs <path>`: Override sysfs root directory (default: `/sys/class/net`).
- `--procfs <path>`: Override procfs root directory (default: `/proc/net`).
- `--resolv <path>`: Override resolv.conf file path (default: `/etc/resolv.conf`).
- `--json`: Output machine-readable JSON envelope: `{"code": <int>, "data": ..., "error": ...}`.

### Invariants (`NCLI1..NCLI6`)

1. **`NCLI1` (Path Hygiene)**: `--sysfs`, `--procfs`, and `--resolv` flags are strictly bounded to $\le 1024$ characters and must not contain control characters. Violations exit with code 2 and emit an audit event.
2. **`NCLI2` (Interface Name Sanitization)**: Interface names are validated against `validate_interface_name` ($\le 15$ chars, `^[a-zA-Z0-9_.-]+$`). Malformed or traversal paths are rejected with exit code 2 (`INVALID_INTERFACE_NAME`).
3. **`NCLI3` (Terminal Output Sanitization)**: Human-readable output is sanitized via `sanitize_terminal` to strip ANSI escape sequences and prevent terminal injection attacks.
4. **`NCLI4` (Deterministic JSON Envelope)**: In `--json` mode, output always follows the standard envelope schema with `code`, `data`, and `error`.
5. **`NCLI5` (Audit Logging & PEP Gating)**: Every execution path (success and failure) emits an audit event via `classify_and_emit` to maintain complete traceability.
6. **`NCLI6` (POSIX Exit Codes)**: Exit code 0 for success, 1 for operational failures / resource not found, and 2 for syntax or validation errors.

### Example Invocations

```bash
# List interfaces in human-readable table
aiosh net list

# Query interface details with JSON envelope
aiosh net show eth0 --json

# View IPv4 routing table
aiosh net routes

# Query DNS resolver configuration
aiosh net dns

# Snapshot full network state
aiosh net state --json

# Hermetic test execution with custom mock paths
aiosh net state --sysfs /tmp/mock/sys/class/net --procfs /tmp/mock/proc/net --resolv /tmp/mock/etc/resolv.conf --json
```

### Constraints & Known Limitations
- Live kernel reads depend on Linux sysfs and procfs structures; on non-Linux platforms (e.g. Windows/macOS), the CLI functions via mock paths or falls back to empty datasets.
- Link state mutation (`up`, `down`) on live systems requires appropriate Linux capabilities (`CAP_NET_ADMIN`) or root privileges.
- IPv6 route parsing is reserved for future milestones; currently `/proc/net/route` handles IPv4 routing tables.

---

## 7. Network Bootstrap MCP Tool Surface (`aios.network.*`)

The MCP tool surface exposes network inspection and control capabilities to autonomous AI agents via the Model Context Protocol over standard JSON-RPC 2.0.

### Tool Registry

| Tool Name | Parameters | Description | Grant Required |
|-----------|------------|-------------|----------------|
| `aios.network.list` | `sysfs_path`, `procfs_path`, `resolv_path`, `grant_id` | List discovered network interfaces with operational state and MAC | No |
| `aios.network.show` | `interface` (required), `sysfs_path`, `procfs_path`, `resolv_path`, `grant_id` | Inspect detailed interface attributes | No |
| `aios.network.routes` | `procfs_path`, `grant_id` | Query host IPv4 routing table | No |
| `aios.network.dns` | `resolv_path`, `grant_id` | Query host DNS nameservers and search domains | No |
| `aios.network.state` | `sysfs_path`, `procfs_path`, `resolv_path`, `grant_id` | Retrieve unified host networking state snapshot | No |
| `aios.network.up` | `interface` (required), `sysfs_path`, `grant_id` | Bring network interface link up (consequential) | No (Audited) |
| `aios.network.down` | `interface` (required), `sysfs_path`, `grant_id` | Bring network interface link down (consequential) | No (Audited) |

### Invariants (`NMCP1..NMCP6`)

1. **`NMCP1` (Tool Schema Completeness)**: Every tool declares a JSON Schema 2020-12 / Draft 7 inputSchema with typed parameters and descriptions.
2. **`NMCP2` (Path Sanitization)**: Path overrides (`sysfs_path`, `procfs_path`, `resolv_path`) are capped at $\le 1024$ characters with control character rejection.
3. **`NMCP3` (Interface Name Validation)**: Interface names must strictly conform to `validate_interface_name` ($\le 15$ chars, `^[a-zA-Z0-9_.-]+$`).
4. **`NMCP4` (PEP & Mutation Controls)**: Link mutations (`up`, `down`) are identified as consequential actions and require valid interface names.
5. **`NMCP5` (Audit Logging)**: Every invocation routes through `dispatch::recorded_call`, writing an immutable audit record with actor, tool name, parameters, and outcome.
6. **`NMCP6` (Deterministic Serialization & Cross-Surface Parity)**: MCP tool outputs match the canonical serialized structures produced by the CLI and core service.

### Example MCP Invocations

```json
// tools/call: aios.network.list
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.network.list",
    "arguments": {}
  }
}

// tools/call: aios.network.show
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.network.show",
    "arguments": {
      "interface": "eth0"
    }
  }
}

// tools/call: aios.network.state
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "aios.network.state",
    "arguments": {}
  }
}
```

### Constraints & Known Limitations
- When running in containers or unprivileged environments, link state changes (`up`, `down`) will fail gracefully if the process lacks `CAP_NET_ADMIN`.
- Path override parameters are intended for hermetic CI testing and containerized testing environments.

---

## 8. Network Bootstrap Configuration Subsystem (`NetworkConfig`)

The Network Bootstrap Configuration Subsystem manages persistent configuration, validation bounds, environment variable ingestion, and default fallbacks for host network bootstrap operations.

### Data Structure

```rust
pub struct NetworkConfig {
    pub default_store_path: PathBuf,       // .aios/network_state.json
    pub sysfs_net_path: PathBuf,           // /sys/class/net
    pub procfs_path: PathBuf,              // /proc/net
    pub resolv_conf_path: PathBuf,         // /etc/resolv.conf
    pub max_interfaces: usize,             // default: 1024, range: 1..10,000
    pub max_routes: usize,                 // default: 4096, range: 1..50,000
    pub max_dns_servers: usize,            // default: 32, range: 1..64
    pub max_payload_bytes: u64,            // default: 10 MB, range: 1024..104,857,600
    pub scan_timeout_secs: u64,            // default: 30s, range: 1..300s
    pub fallback_dns_servers: Vec<String>, // default: ["1.1.1.1", "8.8.8.8"]
}
```

### Invariants (`NCONF1..NCONF6`)

1. **`NCONF1` (Path Hygiene)**:
   - Paths (`default_store_path`, `sysfs_net_path`, `procfs_path`, `resolv_conf_path`) must be non-empty, valid UTF-8, $\le 1024$ characters, contain no ASCII control characters or null bytes (`\0`), and contain no parent directory traversal components (`..`).
2. **`NCONF2` (Capacity Limits)**:
   - `1 <= max_interfaces <= 10,000`
   - `1 <= max_routes <= 50,000`
   - `1 <= max_dns_servers <= 64`
3. **`NCONF3` (Resource & Timeout Bounds)**:
   - `1024 <= max_payload_bytes <= 104,857,600` (1 KB to 100 MB)
   - `1 <= scan_timeout_secs <= 300` (1 to 300 seconds)
4. **`NCONF4` (Fallback DNS Validation)**:
   - Every address in `fallback_dns_servers` must be a valid IPv4 or IPv6 address string parsing into `std::net::IpAddr`.
   - Length of `fallback_dns_servers` must not exceed `max_dns_servers`.
5. **`NCONF5` (Environment Variable Ingestion)**:
   - Ingests `AIOS_NETWORK_CONFIG`, `AIOS_NETWORK_STORE_PATH`, `AIOS_NETWORK_SYSFS_PATH`, `AIOS_NETWORK_PROCFS_PATH`, `AIOS_NETWORK_RESOLV_PATH`, `AIOS_NETWORK_MAX_INTERFACES`, `AIOS_NETWORK_MAX_ROUTES`, `AIOS_NETWORK_MAX_DNS`, `AIOS_NETWORK_TIMEOUT`.
   - Post-validation guard ensures that if any environment variable override causes an invalid state, `from_env()` safely reverts to `NetworkConfig::default()`.
6. **`NCONF6` (Persistence & File Protection)**:
   - Configuration files loaded via `load_from_path` or `from_file` are capped at `MAX_CONFIG_FILE_BYTES` (1 MB) to prevent memory exhaustion DoS.
   - Saves via `save_to_path` use atomic sibling file writes (`.{name}.tmp.{pid}`) followed by atomic filesystem rename, with automatic temporary file cleanup on any error path.

### Environment Variables

| Variable | Description | Default | Bounds |
|---|---|---|---|
| `AIOS_NETWORK_CONFIG` | Path to JSON config file to load | *(None)* | $\le 1024$ chars, valid file |
| `AIOS_NETWORK_STORE_PATH` | Snapshot destination path | `.aios/network_state.json` | $\le 1024$ chars, no `..` |
| `AIOS_NETWORK_SYSFS_PATH` | Sysfs network directory | `/sys/class/net` | $\le 1024$ chars, no `..` |
| `AIOS_NETWORK_PROCFS_PATH` | Procfs network directory | `/proc/net` | $\le 1024$ chars, no `..` |
| `AIOS_NETWORK_RESOLV_PATH` | DNS resolv.conf path | `/etc/resolv.conf` | $\le 1024$ chars, no `..` |
| `AIOS_NETWORK_MAX_INTERFACES` | Maximum interfaces | `1024` | `1..10000` |
| `AIOS_NETWORK_MAX_ROUTES` | Maximum routes | `4096` | `1..50000` |
| `AIOS_NETWORK_MAX_DNS` | Maximum DNS servers | `32` | `1..64` |
| `AIOS_NETWORK_TIMEOUT` | Scan timeout in seconds | `30` | `1..300` |

### Example Configuration (`network_config.json`)

```json
{
  "default_store_path": ".aios/network_state.json",
  "sysfs_net_path": "/sys/class/net",
  "procfs_path": "/proc/net",
  "resolv_conf_path": "/etc/resolv.conf",
  "max_interfaces": 1024,
  "max_routes": 4096,
  "max_dns_servers": 32,
  "max_payload_bytes": 10485760,
  "scan_timeout_secs": 30,
  "fallback_dns_servers": [
    "1.1.1.1",
    "8.8.8.8"
  ]
}
```

### Limitations
- `NetworkConfig` does not manage active dynamic routing protocols (e.g., BGP, OSPF); it governs userspace discovery bounds and persistence parameters.
- Overrides via environment variables apply at process initialization and do not hot-reload running services without a restart or re-instantiation of `NetworkService`.

---

## 9. Automated Testing Architecture & Harness

The Network Bootstrap subsystem provides a multi-layer automated testing framework designed to verify behavior hermetically across the core data model, core service, CLI surface, MCP surface, and configuration subsystem.

### Test Harness Architecture

```
+-----------------------------------------------------------------------+
|                       Mock Filesystem Fixture                         |
|   /sys/class/net/{lo, eth0, wlan0}    /proc/net/route    /etc/resolv  |
+-----------------------------------+-----------------------------------+
                                    |
            +-----------------------+-----------------------+
            |                                               |
            v                                               v
+-----------------------+                       +-----------------------+
|  Rust Automated Suite |                       |  Python Smoke Suite   |
| (test_network_auto.)  |                       | (test_network_e2e_.)  |
+-----------+-----------+                       +-----------+-----------+
            |                                               |
            v                                               v
+-----------------------+                       +-----------------------+
| aiosh-core Discovery  |                       | Cross-Surface Parity  |
| & Config Integration  |                       |  (CLI vs MCP JSON)    |
+-----------------------+                       +-----------------------+
```

### Invariants (`NTEST1..NTEST6`)

1. **`NTEST1` (Hermetic Isolation)**: Automated tests must run exclusively against mock filesystem structures located in temporary directories (`tempfile::TempDir` / `tempfile.TemporaryDirectory`), never modifying or requiring root privileges on host network interfaces.
2. **`NTEST2` (Cross-Surface Parity)**: Network state representations generated by the core service (`NetworkService::get_network_state()`), CLI (`aiosh network state --json`), and MCP (`aios.network.state`) must produce structurally and semantically equivalent JSON documents.
3. **`NTEST3` (Fault & Corrupt Data Injection)**: Suites must inject invalid inputs (corrupted routing table rows, missing sysfs directories, empty resolv.conf files, path traversal interface names) and verify that the system fails gracefully without panicking or leaking descriptors.
4. **`NTEST4` (Audit Trail Integrity)**: Consequential operations (`aios.network.up`, `aios.network.down`) must generate verified audit records with actor attribution, parameters, timestamps, and verdicts.
5. **`NTEST5` (Configuration Integration)**: Test runs must demonstrate that environment variables (`AIOS_NETWORK_*`) successfully redirect mock discovery paths and capacity limits.
6. **`NTEST6` (Deterministic Resource Cleanup)**: All temporary directories, files, and mock sockets must be cleaned up on scope exit (via RAII `Drop` in Rust and `try/finally` context managers in Python).

### Running Automated Test Suites

```bash
# Execute Rust automated integration suite
cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_network_automated

# Execute Python cross-surface end-to-end smoke suite
python code/aiosh-cli/tests/test_network_e2e_smoke.py

# Execute full network regression test suite
python code/aiosh-cli/tests/test_network_smoke.py
python code/aiosh-cli/tests/test_network_service_smoke.py
python code/aiosh-cli/tests/test_network_cli_smoke.py
python code/aiosh-mcp/tests/test_network_mcp_smoke.py
python code/aiosh-cli/tests/test_network_config_smoke.py
```

### Stated Limitations
- Hermetic automated tests simulate sysfs, procfs, and resolv.conf file structures; they do not validate physical NIC firmware, real hardware packet transmission, or active BPF filters.
- Real kernel link state toggling (`ip link set up/down`) is abstracted and tested via filesystem state mutations rather than privileged `ioctl` or Netlink calls in unprivileged CI environments.



