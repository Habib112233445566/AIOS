# Task Evidence: T-01843 - Network Bootstrap / configuration: Scaffold

## Summary
Scaffolded the `NetworkConfig` data structure and API surface within `aiosh-core` (`code/aiosh-rust/aiosh-core/src/network_config.rs`) and registered/re-exported the module in `code/aiosh-rust/aiosh-core/src/lib.rs`.

## Scaffolded Elements
1. **Module Structure**: Created `code/aiosh-rust/aiosh-core/src/network_config.rs`.
2. **Re-export**: Re-exported `pub mod network_config;` and `pub use network_config::NetworkConfig;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
3. **Core Struct**: Defined `pub struct NetworkConfig` with `default_store_path`, `sysfs_net_path`, `procfs_path`, `resolv_conf_path`, `max_interfaces`, `max_routes`, `max_dns_servers`, `max_payload_bytes`, `scan_timeout_secs`, and `fallback_dns_servers`.
4. **Traits & Methods**:
   - `Default` implementation with standard paths (`.aios/network_state.json`, `/sys/class/net`, `/proc/net`, `/etc/resolv.conf`) and standard bounds.
   - `validate(&self) -> Result<(), String>` method signature.
   - `load_from_path(path: &Path) -> Result<Self, String>` method signature.
   - `save_to_path(&self, path: &Path) -> Result<(), String>` method signature.
   - `from_env() -> Self` method signature.

## Verification
- Executed `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core`.
- Zero compilation errors.
