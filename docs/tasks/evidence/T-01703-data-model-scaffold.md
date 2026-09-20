# T-01703: Hardware Detection — Data Model Scaffold

## Metadata
- **Task ID**: `T-01703`
- **Sub-Epic**: Hardware Detection / Data Model
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Scaffolding Engineer**: Antigravity Autonomous Agent

---

## 1. Module Scaffolding
Created `code/aiosh-rust/aiosh-core/src/hardware.rs` and registered `pub mod hardware;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.

Core interfaces established:
1. `DeviceClass` (Cpu, Memory, Block, Network, Gpu, Pci, Usb, System, Other) with `as_str()`.
2. `DeviceBus` (Pci, Usb, Platform, Scsi, Virtio, System, Unknown) with `as_str()`.
3. `HardwareDevice` struct with fields `id`, `name`, `class`, `bus`, `vendor_id`, `device_id`, `vendor_name`, `device_name`, `driver`, `sysfs_path`, `dev_path`, `attributes`, and method `validate()`.
4. `HardwareInventory` struct with fields `timestamp`, `hostname`, `architecture`, `kernel_version`, `devices`, `summary`, and methods `new()`, `add_device()`, `update_summary()`, `validate_invariants()`, `to_json()`, and `from_json()`.
5. Helper validators `validate_device_id`, `validate_hex_id`, and `validate_path`.

---

## 2. Compilation Verification
Executed `cargo check -p aiosh-core`:
```
Checking aiosh-core v0.1.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.24s
```
Zero warnings or errors.

---

## 3. Acceptance Criteria Checklist
- [x] Module file created under `code/aiosh-rust/aiosh-core/src/hardware.rs`.
- [x] Export wired in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- [x] Project compiles cleanly (`cargo check -p aiosh-core`).
- [x] Typed signatures match specification.
