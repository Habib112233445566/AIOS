# T-01713: Hardware Detection — Core Service Scaffold

## Metadata
- **Task ID**: `T-01713`
- **Sub-Epic**: Hardware Detection / Core Service
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Scaffolding Engineer**: Antigravity Autonomous Agent

---

## 1. Scaffold Implementation
Created module `code/aiosh-rust/aiosh-core/src/hardware_service.rs` and registered `pub mod hardware_service;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.

Core structural definitions:
1. `HardwareScanOptions`: Configures optional class filtering (`Option<Vec<DeviceClass>>`) and extended attribute collection flag (`include_attributes`).
2. `HardwareService`: Encapsulates `sysfs_root: PathBuf`, `procfs_root: PathBuf`, and `cached_inventory: RwLock<Option<HardwareInventory>>`.
3. Methods:
   - `HardwareService::new()`: Targets default Linux host mounts (`/sys`, `/proc`).
   - `HardwareService::with_roots(sysfs_root, procfs_root)`: Enables custom sysfs/procfs root paths for hermetic, cross-platform unit testing.
   - `HardwareService::scan(&self, options)`: Discovery entrypoint.
   - `HardwareService::get_cached_inventory(&self)`: Cached inventory access.
   - `HardwareService::invalidate_cache(&self)`: Cache invalidation.

---

## 2. Compilation Verification
Executed `cargo check -p aiosh-core`:
```
Checking aiosh-core v0.1.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 41.34s
```
Status: PASS (0 compile errors).

---

## 3. Acceptance Criteria Checklist
- [x] Source file created at `code/aiosh-rust/aiosh-core/src/hardware_service.rs`.
- [x] Registered and exported in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- [x] `cargo check -p aiosh-core` compiles cleanly.
- [x] New interfaces exist and match specification.
