# T-01741: Hardware Detection — Configuration Research

## Metadata
- **Task ID**: `T-01741`
- **Sub-Epic**: Sub-Epic 5: Hardware Detection Configuration
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Research & Analysis

### 1.1 Configuration Subsystem Patterns in AIOS
Examining existing configuration subsystems in `aiosh-core` (`kernel_module_config.rs`, `package_config.rs`, `distro_config.rs`, `session_config.rs`):
1. **Strong Typing & Serialization**: Configuration structures derive `Debug, Clone, PartialEq, Eq, Serialize, Deserialize` and provide a deterministic `Default` implementation.
2. **Validation Method (`validate(&self) -> Result<(), String>`)**: Checks all paths for length bounds ($\le 1024$ bytes) and control character rejection, ensures numeric thresholds are within safety envelopes, and verifies non-empty identifiers.
3. **Multi-Tier Resolution (`from_env` / `from_file`)**:
   - Tier 1: Explicit path / programmatic builder.
   - Tier 2: Environment variable override (`AIOSH_HARDWARE_CONFIG`, `AIOSH_HARDWARE_SYSFS`, etc.).
   - Tier 3: Default configuration (`HardwareConfig::default()`).
4. **Resilient Loading & Safe Defaults**: If the file does not exist, returns `Default::default()`; if the file is invalid, returns descriptive validation errors without panicking.

### 1.2 Hardware Detection Configuration Requirements
The `HardwareConfig` structure must manage:
- `default_store_path`: Canonical path to persisted hardware inventory JSON (`.aios/hardware_inventory.json`).
- `sysfs_path`: System sysfs mount point (default `/sys`).
- `procfs_path`: System procfs mount point (default `/proc`).
- `enabled_classes`: Optional filter of `DeviceClass` variants to scan (None = scan all classes).
- `include_attributes`: Boolean toggle to collect or omit detailed sysfs key-value attributes.
- `max_devices`: Maximum device limit per inventory manifest (default 10,000).
- `max_payload_bytes`: Upper bound on JSON serialized inventory size (default 10 MB).
- `scan_timeout_secs`: Maximum scan duration before timeout (default 30 seconds, bounded between 1 and 300).

---

## 2. Technical Findings for Implementation
1. **Module Placement**: `code/aiosh-rust/aiosh-core/src/hardware_config.rs` and re-exported in `aiosh-core/src/lib.rs`.
2. **Service Integration**: Extend `HardwareService` with `HardwareService::with_config(config: HardwareConfig)` and `HardwareService::from_env()`.
3. **No Unsafe Code**: All configuration file reads and parsing utilize standard library `fs` and `serde_json` with strict bounds.
