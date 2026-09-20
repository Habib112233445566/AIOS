# T-01757: Hardware Detection — Automated Tests Security Review

## Metadata
- **Task ID**: `T-01757`
- **Sub-Epic**: Sub-Epic 6: Hardware Detection Automated Tests
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Security Review Scope
Reviewed `code/aiosh-rust/aiosh-core/tests/test_hardware_automated.rs` and `code/aiosh-cli/tests/test_hardware_automated_smoke.py` focusing on test fixture containment, temporary directory cleanup, symlink security, and scale test resource bounds.

---

## 2. Threat Analysis & Vulnerability Findings

### Finding AT-SEC-1: Temporary Directory Cleanup Guarantee
- **Vulnerability**: `MockSysfsBuilder` holds a `tempfile::TempDir`. If test execution panics or aborts uncleanly, temporary directories could accumulate in OS temp directories.
- **Severity**: **LOW** (CWE-459)
- **Mitigation**: Confirmed `TempDir` implements RAII `Drop` which removes directories on test completion or normal unwinding. In `T-01758`, ensure builder methods do not expose raw paths without ownership.

### Finding AT-SEC-2: Symlink Escape Vectors in Probers
- **Vulnerability**: Probers resolve `driver` symlinks in `sysfs_root/bus/pci/devices/*/driver`. If a test or attacker-crafted sysfs tree creates a symlink pointing to an absolute host path (e.g. `/etc/passwd`), does the prober leak file content?
- **Analysis**: In `code/aiosh-rust/aiosh-core/src/hardware_service.rs`, `resolve_driver_name` only invokes `p.file_name().map(|n| n.to_string_lossy().to_string())`. It never opens or reads content from the symlink target.
- **Severity**: **INFORMATIONAL** (Safe by design)
- **Mitigation**: Add an explicit test case in `T-01758` verifying that an external-pointing symlink only resolves the terminal filename and does not traverse or leak host data.

### Finding AT-SEC-3: Scale Test Directory Count Bound
- **Vulnerability**: Scale tests creating $> 1,000$ directories can introduce I/O latency on Windows filesystems with real-time antivirus scanning.
- **Severity**: **LOW** (CWE-400)
- **Mitigation**: Ensure scale tests create the minimum entries required to exceed `MAX_PROBE_ENTRIES = 1024` (e.g. 1,050 entries instead of unnecessarily large counts).

---

## 3. Hardening Recommendations for T-01758
1. Add an explicit test verifying symlink traversal safety in `MockSysfsBuilder`.
2. Optimize scale test entry count to 1,050 entries for faster, reliable execution under 5 seconds.
