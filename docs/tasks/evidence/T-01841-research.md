# T-01841: Network Bootstrap / Configuration: Research

## 1. Overview
- **Task ID**: `T-01841`
- **Sub-Epic**: 5 (Configuration)
- **Goal**: Research configuration management, environment variables, validation rules, and persistence for Network Bootstrap.

---

## 2. Research Findings: Facts vs Assumptions

### A. Authoritative Facts
1. **Existing AIOS Configuration Architecture**:
   - Subsystem configs (e.g. `HardwareConfig`, `KernelModuleConfig`, `DistroConfig`) are located in `code/aiosh-rust/aiosh-core/src/*_config.rs`.
   - Each config struct derives `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`.
   - Each config implements `Default`, `validate()`, `from_env()`, `from_file()`, and `save_to_path()`.
   - Invariant validation rejects:
     - Path length > 1024 characters.
     - Non-UTF-8 or empty paths.
     - ASCII/Unicode control characters (`c.is_control()`).
     - Directory traversal sequences (`..`).
   - Size ceilings:
     - Policy/config files capped at 1 MB.
     - Payloads capped at 10 MB.
     - Scan timeouts capped at 300 seconds.
2. **Network Bootstrap Configuration Requirements**:
   - Default paths for sysfs net root (`/sys/class/net`), procfs net root (`/proc/net`), and resolv.conf (`/etc/resolv.conf`).
   - Default persistence path for serialized network state snapshot (`.aios/network_state.json`).
   - Collection bounds for discovered interfaces, routes, and DNS servers.
   - Fallback DNS server configuration when resolv.conf is empty or unavailable.

### B. Assumptions
1. Environment variable prefix: `AIOS_NETWORK_*`
   - `AIOS_NETWORK_STORE_PATH`
   - `AIOS_NETWORK_SYSFS_PATH`
   - `AIOS_NETWORK_PROCFS_PATH`
   - `AIOS_NETWORK_RESOLV_PATH`
   - `AIOS_NETWORK_MAX_INTERFACES`
   - `AIOS_NETWORK_MAX_ROUTES`
   - `AIOS_NETWORK_TIMEOUT`
2. Fallback DNS defaults: `["1.1.1.1", "8.8.8.8"]`.

---

## 3. Configuration Invariants (`NCONF1..NCONF6`)
- **`NCONF1` (Path Hygiene)**: All file and directory paths must be valid UTF-8, non-empty, $\le 1024$ characters, contain no control characters, and contain no `..` traversal components.
- **`NCONF2` (Collection Caps)**: `max_interfaces` bounded to $[1, 10_000]$, `max_routes` bounded to $[1, 50_000]$, and `max_dns_servers` bounded to $[1, 64]$.
- **`NCONF3` (Payload & Timeout Bounds)**: `max_payload_bytes` in $[1024, 104_857_600]$ (1 KB to 100 MB), `scan_timeout_secs` in $[1, 300]$.
- **`NCONF4` (Fallback DNS Validation)**: Fallback DNS servers must be valid IPv4 or IPv6 address strings.
- **`NCONF5` (Environment Override Precedence)**: Environment variables override defaults; malformed env values fail-safe with explicit validation errors or fallback.
- **`NCONF6` (Atomic Persistence & Roundtrip)**: `save_to_path` writes to a PID-scoped temporary file and atomically renames to the target path.

---

## 4. Decisions
- Implement `NetworkConfig` in `code/aiosh-rust/aiosh-core/src/network_config.rs` following the `HardwareConfig` pattern.
- Re-export `NetworkConfig` from `aiosh-core/src/lib.rs`.
