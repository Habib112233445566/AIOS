# T-01752: Hardware Detection — Automated Tests Specification

## Metadata
- **Task ID**: `T-01752`
- **Sub-Epic**: Sub-Epic 6: Hardware Detection Automated Tests
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Formal Invariants (AT1..AT5)

| Invariant | Name | Formal Definition | Verification Mechanism |
| :--- | :--- | :--- | :--- |
| **AT1** | Hermetic Isolation | $\forall t \in \text{Tests}: \text{roots}(t) \subset \text{TempDir} \land \neg \text{access}(\text{"/sys"}, \text{"/proc"})$ | Assert custom sysfs/procfs roots are within ephemeral temp directories. |
| **AT2** | Fault Injection Robustness | Corrupt, partial, or missing files $\implies \text{graceful degradation} \land \neg \text{panic}$ | Negative fault injection tests (corrupt hex, empty files, broken symlinks). |
| **AT3** | Deterministic Identification | $\forall d \in \text{Discovered}: d.\text{id}.\text{is\_canonical}() \land d.\text{class} == \text{expected}(d)$ | Golden fixture tests asserting exact normalized IDs and classes. |
| **AT4** | Invariant Compliance | $\forall \text{inv} \in \text{ScanResults}: \text{validate\_invariants}(\text{inv}) == \text{Ok}(())$ | Full invariant suite execution on synthetic inventories. |
| **AT5** | Scale & Traversal Bound | $|\text{entries}| > 1024 \implies |\text{inspected}| \le 1024 \land \text{time} < 500\text{ms}$ | Large fixture tests with 1,200 synthetic device entries. |

---

## 2. Test Harness Contract (`MockSysfsBuilder`)

```rust
pub struct MockSysfsBuilder {
    temp_dir: tempfile::TempDir,
}

impl MockSysfsBuilder {
    pub fn new() -> Self;
    pub fn add_pci(&mut self, slot: &str, vendor: &str, device: &str, class_code: &str, driver: Option<&str>) -> &mut Self;
    pub fn add_usb(&mut self, id: &str, vendor: &str, product: &str, manufacturer: &str, prod_name: &str) -> &mut Self;
    pub fn add_block(&mut self, name: &str, size_sectors: u64, rotational: bool, model: &str) -> &mut Self;
    pub fn add_net(&mut self, name: &str, mac: &str, speed: i32, operstate: &str) -> &mut Self;
    pub fn add_cpu(&mut self, cpu_id: usize, model: &str, mhz: f64) -> &mut Self;
    pub fn add_dmi(&mut self, vendor: &str, product: &str, version: &str) -> &mut Self;
    pub fn roots(&self) -> (PathBuf, PathBuf); // (sysfs_root, procfs_root)
}
```

---

## 3. Test Matrix Definition
1. `test_at1_hermetic_isolation`: Runs full scan targeting empty mock root; verifies zero devices discovered and no access to host `/sys`.
2. `test_at2_fault_injection_corrupted_pci`: Writes malformed vendor/device/class files; verifies discovery ignores malformed files without panicking.
3. `test_at2_fault_injection_broken_symlinks`: Points driver symlink to missing path; verifies driver is reported as `None`.
4. `test_at3_deterministic_classification`: Validates all 9 device classes across synthetic PCI, USB, Block, Net, CPU, and DMI devices.
5. `test_at4_invariant_compliance`: Asserts `inv.validate_invariants()` passes on comprehensive synthetic inventory.
6. `test_at5_scale_and_traversal_bound`: Generates 1,200 mock PCI devices; asserts exactly 1,024 entries are processed (`MAX_PROBE_ENTRIES`).
