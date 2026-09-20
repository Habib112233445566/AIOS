# Task Evidence: T-01844 - Network Bootstrap / configuration: Implementation

## Summary
Fully implemented `NetworkConfig` in `code/aiosh-rust/aiosh-core/src/network_config.rs` adhering to all specified invariants `NCONF1..NCONF6`.

## Invariant Enforcement & Implementation Details
1. **`NCONF1` (Path Hygiene)**:
   - Validates `default_store_path`, `sysfs_net_path`, `procfs_path`, and `resolv_conf_path`.
   - Rejects non-UTF-8 paths, empty/whitespace-only paths, paths exceeding 1024 characters, paths with ASCII control characters or null bytes, and paths containing parent directory traversal (`ParentDir` / `..`).
2. **`NCONF2` (Capacity Limits)**:
   - `max_interfaces`: Enforces range `1..=10,000`.
   - `max_routes`: Enforces range `1..=50,000`.
   - `max_dns_servers`: Enforces range `1..=64`.
3. **`NCONF3` (Resource & Timeout Bounds)**:
   - `max_payload_bytes`: Enforces range `1024..=104_857_600` (1 KB to 100 MB).
   - `scan_timeout_secs`: Enforces range `1..=300`.
4. **`NCONF4` (Fallback DNS Validation)**:
   - Ensures `fallback_dns_servers.len() <= max_dns_servers`.
   - Validates each DNS string parses into a valid `std::net::IpAddr`.
   - Default fallbacks: `["1.1.1.1", "8.8.8.8"]`.
5. **`NCONF5` (Environment Ingestion)**:
   - `from_env()` ingests:
     - `AIOS_NETWORK_CONFIG` (file load path)
     - `AIOS_NETWORK_STORE_PATH` / `AIOS_NETWORK_STORE`
     - `AIOS_NETWORK_SYSFS_PATH` / `AIOS_NETWORK_SYSFS`
     - `AIOS_NETWORK_PROCFS_PATH` / `AIOS_NETWORK_PROCFS`
     - `AIOS_NETWORK_RESOLV_PATH` / `AIOS_NETWORK_RESOLV_CONF`
     - `AIOS_NETWORK_MAX_INTERFACES`
     - `AIOS_NETWORK_MAX_ROUTES`
     - `AIOS_NETWORK_MAX_DNS`
     - `AIOS_NETWORK_TIMEOUT` / `AIOS_NETWORK_TIMEOUT_SECS`
   - Post-validation fallback ensures invalid environment values do not put the system into an invalid configuration state.
6. **`NCONF6` (Persistence & File Protection)**:
   - `load_from_path` / `from_file`: Enforces `MAX_CONFIG_FILE_BYTES = 1,048,576` (1 MB limit) to prevent unbounded file reads/memory exhaustion.
   - `save_to_path`: Validates before writing, creates parent directories if needed, writes atomically using a sibling temp file (`.{name}.tmp.{pid}`), and renames atomically.

## Verification
- Compilation verified via `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core`.
- Passed with 0 errors.
