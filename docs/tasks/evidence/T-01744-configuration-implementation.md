# T-01744: Hardware Detection — Configuration Implementation

## Metadata
- **Task ID**: `T-01744`
- **Sub-Epic**: Sub-Epic 5: Hardware Detection Configuration
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Implementation Summary
Implemented the full configuration lifecycle for the Hardware Detection subsystem in `code/aiosh-rust/aiosh-core/src/hardware_config.rs` and integrated it with `HardwareService` in `code/aiosh-rust/aiosh-core/src/hardware_service.rs`.

---

## 2. Invariant Enforcement (HCFG1..HCFG5)
1. **HCFG1 (Path Hygiene)**:
   - Validates `default_store_path`, `sysfs_path`, and `procfs_path` against emptiness, max length (1024 bytes), and control/NUL characters.
2. **HCFG2 (Class Filtering & Uniqueness)**:
   - Limits `enabled_classes` count to $\le 9$.
   - Uses `BTreeSet` to reject duplicate `DeviceClass` entries.
3. **HCFG3 (Resource Bounds)**:
   - Enforces $1 \le \text{max\_devices} \le 50,000$.
   - Enforces $1024 \le \text{max\_payload\_bytes} \le 104,857,600$.
4. **HCFG4 (Timeout Bounds)**:
   - Enforces $1 \le \text{scan\_timeout\_secs} \le 300$.
5. **HCFG5 (Lossless Serialization & Fallback)**:
   - `load_from_path()` safely falls back to `HardwareConfig::default()` if the file is missing.
   - `save_to_path()` validates configuration before writing and creates parent directories if needed.
   - `from_env()` reads and respects environment variables `AIOSH_HARDWARE_CONFIG`, `AIOSH_HARDWARE_SYSFS`, `AIOSH_HARDWARE_PROCFS`, `AIOSH_HARDWARE_STORE`, `AIOSH_HARDWARE_INCLUDE_ATTRS`, and `AIOSH_HARDWARE_TIMEOUT_SECS`.

---

## 3. Integration with HardwareService
- Added `HardwareService::with_config(config: &HardwareConfig) -> Self`.
- Added `HardwareService::scan_with_config(&self, config: &HardwareConfig) -> Result<HardwareInventory, String>`.
