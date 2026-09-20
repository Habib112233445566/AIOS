# T-01751: Hardware Detection — Automated Tests Research

## Metadata
- **Task ID**: `T-01751`
- **Sub-Epic**: Sub-Epic 6: Hardware Detection Automated Tests
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Research Objectives
Investigated automated testing architectures, fault-injection strategies, and synthetic fixture generation for host hardware discovery across heterogeneous environments (bare metal, containerized CI, Windows dev hosts, QEMU).

---

## 2. Key Findings & Design Considerations

### 2.1 Cross-Platform Mock Sysfs Generation
- **Filesystem Constraints**: On Windows hosts, colon characters (`:`) in directory names (e.g. PCI bus IDs like `0000:00:02.0`) trigger `ERROR_INVALID_NAME` (code 123).
- **Subsystem Translation**: The prober subsystem already contains built-in translation (`0000_00_02.0` $\to$ `0000:00:02.0`). Automated test fixtures must use underscores for directory creation while asserting colon-normalized IDs in generated `HardwareInventory` models.

### 2.2 Fault Injection Matrix
Automated test suites must exercise resilience against corrupted or abnormal sysfs/procfs files:
1. **Truncated / Corrupt Hex**: `vendor` containing `"0x"` without digits, or non-hex string `"invalid"`.
2. **Missing Files**: Devices missing `device` or `class` attributes.
3. **Broken Symlinks**: `driver` symlinks pointing to non-existent targets.
4. **Unbounded Traversal**: Directories with $> 1,024$ entries to assert strict enforcement of `MAX_PROBE_ENTRIES`.
5. **Partial Subsystem Trees**: Scenarios where only a subset of `/sys` directories exist (e.g. container environments without `/sys/bus/pci`).

### 2.3 Performance & Scale Benchmarking
- The test harness should construct synthetic hierarchies containing hundreds of devices across all classes (CPU, Memory, Block, Network, GPU, PCI, USB, System) and verify discovery completes within $< 500$ ms with deterministic sorting and zero invariant violations.

---

## 3. Recommended Specification for T-01752
Formulate invariants `AT1..AT5`:
- `AT1`: Hermetic Mock Isolation (independent `TempDir` fixtures).
- `AT2`: Fault Injection Robustness (graceful handling of corrupted/truncated files).
- `AT3`: Deterministic Classification & Identification (normalized IDs, correct class mapping).
- `AT4`: Invariant Compliance (`HD1..HD5`, `HS1..HS5`, `HCFG1..HCFG5`).
- `AT5`: Scale & Bound Enforcement (1,024 directory limit, 10,000 device ceiling).
