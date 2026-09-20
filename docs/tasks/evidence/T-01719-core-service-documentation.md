# T-01719: Hardware Detection — Core Service Documentation

## Metadata
- **Task ID**: `T-01719`
- **Sub-Epic**: Hardware Detection / Core Service
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Documentation Scope
Authored Section 8 ("Hardware Detection Core Service") in `docs/hardware_detection.md`:
1. **Discovery Engine Architecture**:
   - `HardwareService` API, constructors (`new`, `with_roots`), and configuration via `HardwareScanOptions`.
2. **Subsystem Prober Technical Specifications**:
   - Detailed operational specification for PCI (`probe_pci`), USB (`probe_usb`), Block (`probe_block`), Network (`probe_net`), CPU (`probe_cpu`), and DMI/SMBIOS (`probe_system`).
3. **Invariants Matrix (HS1..HS5)**:
   - Formally documented mathematical invariants HS1 (Graceful Degradation), HS2 (Accurate Classification), HS3 (Deterministic Ordering), HS4 (Input Sanitization), and HS5 (Inventory Integrity) with mapped automated verification tests.
4. **Task Evidence Index**:
   - Complete hyperlinked traceability matrix for tasks `T-01711` through `T-01720`.

---

## 2. Acceptance Criteria Checklist
- [x] Section 8 added to master reference `docs/hardware_detection.md`.
- [x] Code snippets for Rust usage documented.
- [x] All 6 subsystem probers explained.
- [x] HS1..HS5 invariant mapping recorded.
