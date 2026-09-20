# T-01714: Hardware Detection — Core Service Implementation

## Metadata
- **Task ID**: `T-01714`
- **Sub-Epic**: Hardware Detection / Core Service
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Implementation Architecture
Implemented the complete discovery engine in `code/aiosh-rust/aiosh-core/src/hardware_service.rs`:

1. **Subsystem Probers**:
   - `probe_pci`: Scans `<sysfs>/bus/pci/devices/`, parses `vendor`, `device`, PCI `class` (maps `0x03` to GPU, `0x01` to Block, `0x02` to Network, else PCI), and resolves active driver symlinks.
   - `probe_usb`: Scans `<sysfs>/bus/usb/devices/`, parses `idVendor`, `idProduct`, `manufacturer`, `product`, `speed`.
   - `probe_block`: Scans `<sysfs>/class/block/`, parses `size`, `queue/rotational`, `removable`, and `device/model`.
   - `probe_net`: Scans `<sysfs>/class/net/`, parses `address` (MAC), `operstate`, and `speed`.
   - `probe_cpu`: Scans `/proc/cpuinfo` and `/sys/devices/system/cpu/`, computing core counts and CPU model strings.
   - `probe_system`: Scans `/sys/class/dmi/id/`, extracting `sys_vendor`, `product_name`, `bios_version`, and `chassis_type`.
2. **Orchestration & Sanitization**:
   - `HardwareService::scan`: Aggregates all probers, deduplicates by ID, applies optional `HardwareScanOptions.classes` filters, applies `include_attributes` configuration, sorts deterministically by device ID (HS3), builds `HardwareInventory`, and enforces invariants HD1..HD5.
3. **Cross-Platform / Hermetic Mock Testing**:
   - `HardwareService::with_roots(sysfs_root, procfs_root)` enables mocking arbitrary hardware topologies via filesystem fixtures on Windows, macOS, or Linux without requiring real hardware.
4. **Safety & Bounds**:
   - `MAX_PROBE_ENTRIES = 1024`: Bounds directory scans.
   - `read_trimmed_file` caps strings to 1024 bytes.
   - `normalize_hex_id` strips `0x` prefixes and strictly enforces 4-digit lowercase hex.

---

## 2. Compilation Verification
Executed `cargo check -p aiosh-core`:
```
Checking aiosh-core v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-core)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.75s
```
Status: PASS (0 errors, 0 warnings).

---

## 3. Acceptance Criteria Checklist
- [x] Full prober implementation for PCI, USB, Block, Net, CPU, and System.
- [x] Hermetic mock roots supported via `with_roots`.
- [x] Bounded iteration and safe string normalization.
- [x] Invariants HD1..HD5 validated on every returned inventory.
