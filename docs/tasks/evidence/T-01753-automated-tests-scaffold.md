# T-01753: Hardware Detection — Automated Tests Scaffold

## Metadata
- **Task ID**: `T-01753`
- **Sub-Epic**: Sub-Epic 6: Hardware Detection Automated Tests
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Scaffold Overview
Created the automated test harness `MockSysfsBuilder` and structured test scenarios in `code/aiosh-rust/aiosh-core/tests/test_hardware_automated.rs`.

---

## 2. Scaffold Components
- **`MockSysfsBuilder`**:
  - Ephemeral `TempDir` management for isolated sysfs and procfs trees.
  - Builder methods: `add_pci`, `add_usb`, `add_block`, `add_net`, `add_cpu`, `add_dmi`.
  - Path provider `roots()` returning `(&Path, &Path)`.
- **Test Scenarios**:
  - `test_at1_hermetic_isolation`
  - `test_at2_fault_injection_corrupted_pci`
  - `test_at3_deterministic_classification`
  - `test_at4_invariant_compliance`
  - `test_at5_scale_and_traversal_bound`
