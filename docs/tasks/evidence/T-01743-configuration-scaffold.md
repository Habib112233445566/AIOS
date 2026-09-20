# T-01743: Hardware Detection — Configuration Scaffold

## Metadata
- **Task ID**: `T-01743`
- **Sub-Epic**: Sub-Epic 5: Hardware Detection Configuration
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Overview
Scaffolded the strongly typed configuration structure `HardwareConfig` in `code/aiosh-rust/aiosh-core/src/hardware_config.rs` and registered it within `code/aiosh-rust/aiosh-core/src/lib.rs`.

---

## 2. Scaffold Implementation Details
- **Module**: `aiosh_core::hardware_config`
- **Primary Type**: `HardwareConfig`
- **Fields**:
  - `default_store_path: PathBuf` (`.aios/hardware_inventory.json`)
  - `sysfs_path: PathBuf` (`/sys`)
  - `procfs_path: PathBuf` (`/proc`)
  - `enabled_classes: Option<Vec<DeviceClass>>` (`None`)
  - `include_attributes: bool` (`true`)
  - `max_devices: usize` (`10_000`)
  - `max_payload_bytes: u64` (`10,485,760` — 10 MB)
  - `scan_timeout_secs: u64` (`30`)
- **Traits Derived**: `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`
- **Methods Defined**:
  - `default() -> Self`
  - `validate(&self) -> Result<(), String>`
  - `load_from_path(path: &Path) -> Result<Self, String>`
  - `save_to_path(&self, path: &Path) -> Result<(), String>`
  - `from_env() -> Self`

---

## 3. Module Registration
Registered in `code/aiosh-rust/aiosh-core/src/lib.rs`:
```rust
pub mod hardware_config;
pub use hardware_config::HardwareConfig;
```
