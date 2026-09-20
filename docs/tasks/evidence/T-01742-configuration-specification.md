# T-01742: Hardware Detection — Configuration Specification

## Metadata
- **Task ID**: `T-01742`
- **Sub-Epic**: Sub-Epic 5: Hardware Detection Configuration
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Mathematical & Structural Invariants (HCFG1..HCFG5)

| Invariant | Name | Formal Guarantee | Verification Criteria |
| :--- | :--- | :--- | :--- |
| **HCFG1** | Path Hygiene | $\forall p \in \{\text{store}, \text{sysfs}, \text{procfs}\}: p \neq \text{""} \land \text{len}(p) \le 1024 \land \neg \exists c \in p: c.\text{is\_control}()$. | Path bound and control character rejection tests. |
| **HCFG2** | Class Filtering | If `Some(classes)`, $\forall c \in \text{classes}: c \in \text{DeviceClass} \land \text{unique}(c) \land \text{len} \le 9$. | Filter parsing and duplicate class rejection. |
| **HCFG3** | Resource Bounds | $1 \le \text{max\_devices} \le 50,000 \land 1024 \le \text{max\_payload\_bytes} \le 104,857,600$. | Bounds validation tests. |
| **HCFG4** | Timeout Bounds | $1 \le \text{scan\_timeout\_secs} \le 300$. | Timeout validation tests. |
| **HCFG5** | Lossless Serialization & Resilience | $\text{from\_json}(\text{to\_json}(\text{cfg})) == \text{cfg}$; missing files return `default()`. | JSON roundtrip and missing file tests. |

---

## 2. Configuration Data Contract

```rust
pub struct HardwareConfig {
    pub default_store_path: PathBuf,
    pub sysfs_path: PathBuf,
    pub procfs_path: PathBuf,
    pub enabled_classes: Option<Vec<DeviceClass>>,
    pub include_attributes: bool,
    pub max_devices: usize,
    pub max_payload_bytes: u64,
    pub scan_timeout_secs: u64,
}
```

### 2.1 Default Values
- `default_store_path`: `PathBuf::from(".aios/hardware_inventory.json")`
- `sysfs_path`: `PathBuf::from("/sys")`
- `procfs_path`: `PathBuf::from("/proc")`
- `enabled_classes`: `None` (all classes enabled)
- `include_attributes`: `true`
- `max_devices`: `10,000`
- `max_payload_bytes`: `10,485,760` (10 MB)
- `scan_timeout_secs`: `30`

### 2.2 Environment Variable Mappings
- `AIOSH_HARDWARE_CONFIG`: Path to JSON configuration file.
- `AIOSH_HARDWARE_SYSFS`: Overrides `sysfs_path`.
- `AIOSH_HARDWARE_PROCFS`: Overrides `procfs_path`.
- `AIOSH_HARDWARE_STORE`: Overrides `default_store_path`.
- `AIOSH_HARDWARE_INCLUDE_ATTRS`: Overrides `include_attributes` (`"1"|"true"|"0"|"false"`).
- `AIOSH_HARDWARE_TIMEOUT_SECS`: Overrides `scan_timeout_secs` (parsed as `u64`).

---

## 3. Serialization Schema Example
```json
{
  "default_store_path": ".aios/hardware_inventory.json",
  "sysfs_path": "/sys",
  "procfs_path": "/proc",
  "enabled_classes": ["cpu", "gpu", "block", "network"],
  "include_attributes": true,
  "max_devices": 10000,
  "max_payload_bytes": 10485760,
  "scan_timeout_secs": 30
}
```
