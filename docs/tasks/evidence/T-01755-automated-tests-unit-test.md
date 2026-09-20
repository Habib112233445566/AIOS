# T-01755: Hardware Detection — Automated Tests Unit Test

## Metadata
- **Task ID**: `T-01755`
- **Sub-Epic**: Sub-Epic 6: Hardware Detection Automated Tests
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Test Execution & Coverage
Executed automated unit test suite in `code/aiosh-rust/aiosh-core/tests/test_hardware_automated.rs` testing invariants AT1..AT5:
- `test_at1_hermetic_isolation`: Confirms empty mock root produces empty inventory without host leakage.
- `test_at2_fault_injection_corrupted_pci`: Confirms malformed vendor codes (`0xZZZZ`, `0x`) do not panic and resolve to `None`.
- `test_at2_fault_injection_missing_attributes`: Confirms devices missing optional sysfs attributes (e.g. `size`, `rotational`, `speed`) are discovered safely.
- `test_at3_deterministic_classification`: Confirms accurate classification for GPU, Block, Network, USB, CPU, and System devices.
- `test_at3_filtering_and_attribute_stripping`: Confirms class whitelist filtering and attribute stripping work correctly.
- `test_at4_invariant_compliance`: Confirms `HD1..HD5` invariant validation and `HS3` deterministic sorting on synthetic inventory.
- `test_at5_scale_and_traversal_bound`: Confirms 1,100 mock PCI devices are capped at `MAX_PROBE_ENTRIES = 1024` and scan completes in $< 500$ ms.
