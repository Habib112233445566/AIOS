# T-01842: Network Bootstrap / Configuration: Specification

## 1. Overview
- **Task ID**: `T-01842`
- **Sub-Epic**: 5 (Configuration)
- **Goal**: Formally specify the `NetworkConfig` data structure, validation rules, environment variable ingestion, and persistence contracts.

---

## 2. Specification: Data Structure & Defaults

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Path to persisted network state snapshot JSON (default: .aios/network_state.json)
    pub default_store_path: PathBuf,
    /// Path to sysfs net directory (default: /sys/class/net)
    pub sysfs_net_path: PathBuf,
    /// Path to procfs net directory (default: /proc/net)
    pub procfs_path: PathBuf,
    /// Path to resolv.conf file (default: /etc/resolv.conf)
    pub resolv_conf_path: PathBuf,
    /// Max interfaces to discover (default: 1024, range: 1..10,000)
    pub max_interfaces: usize,
    /// Max routes to parse (default: 4096, range: 1..50,000)
    pub max_routes: usize,
    /// Max DNS servers to collect (default: 32, range: 1..64)
    pub max_dns_servers: usize,
    /// Max serialized document payload bytes (default: 10 MB, range: 1024..104,857,600)
    pub max_payload_bytes: u64,
    /// Timeout in seconds for discovery operations (default: 30, range: 1..300)
    pub scan_timeout_secs: u64,
    /// Fallback DNS nameservers if resolv.conf is missing or empty (default: ["1.1.1.1", "8.8.8.8"])
    pub fallback_dns_servers: Vec<String>,
}
```

---

## 3. Validation Contract (`validate(&self) -> Result<(), String>`)

1. **`NCONF1` (Path Hygiene)**:
   - For `default_store_path`, `sysfs_net_path`, `procfs_path`, `resolv_conf_path`:
     - Must be valid UTF-8 and non-empty.
     - Maximum 1024 characters.
     - Must not contain control characters or null bytes (`\0`).
     - Must not contain parent directory traversal (`..`).
2. **`NCONF2` (Collection Caps)**:
   - `1 <= max_interfaces <= 10,000`
   - `1 <= max_routes <= 50,000`
   - `1 <= max_dns_servers <= 64`
3. **`NCONF3` (Payload & Timeout Bounds)**:
   - `1024 <= max_payload_bytes <= 104,857,600` (1 KB to 100 MB)
   - `1 <= scan_timeout_secs <= 300` (1 to 300 seconds)
4. **`NCONF4` (Fallback DNS Validation)**:
   - Every IP string in `fallback_dns_servers` must parse into a valid `std::net::IpAddr`.
   - Cannot contain more than 64 fallback servers.

---

## 4. Environment Ingestion (`from_env() -> Self`)

| Environment Variable | Target Field | Parsing & Fallback |
|----------------------|--------------|-------------------|
| `AIOS_NETWORK_STORE_PATH` | `default_store_path` | Non-empty string |
| `AIOS_NETWORK_SYSFS_PATH` | `sysfs_net_path` | Non-empty string |
| `AIOS_NETWORK_PROCFS_PATH` | `procfs_path` | Non-empty string |
| `AIOS_NETWORK_RESOLV_PATH` | `resolv_conf_path` | Non-empty string |
| `AIOS_NETWORK_MAX_INTERFACES` | `max_interfaces` | `usize::from_str` |
| `AIOS_NETWORK_MAX_ROUTES` | `max_routes` | `usize::from_str` |
| `AIOS_NETWORK_MAX_DNS` | `max_dns_servers` | `usize::from_str` |
| `AIOS_NETWORK_TIMEOUT` | `scan_timeout_secs` | `u64::from_str` |

---

## 5. Persistence Contract

- **`from_file(path)`**: Reads file up to 1 MB (`MAX_CONFIG_FILE_BYTES = 1,048,576`), parses JSON, and invokes `validate()`.
- **`save_to_path(path)`**:
  - Invokes `validate()`.
  - Serializes to canonical formatted JSON.
  - Writes to `<path>.tmp.<pid>` with permissions `0600` (Unix).
  - Atomically renames to target `<path>`.
