# T-01702: Hardware Detection — Data Model Specification

## Metadata
- **Task ID**: `T-01702`
- **Sub-Epic**: Hardware Detection / Data Model
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Specification Engineer**: Antigravity Autonomous Agent

---

## 1. Scope & System Role
The Hardware Detection data model (`code/aiosh-rust/aiosh-core/src/hardware.rs`) establishes the formal Rust types, serialization formats, validation rules, and error handling for introspecting and cataloging host hardware in AIOS.

---

## 2. Type Definitions & Enums

### 2.1 `DeviceClass` (Enum)
Categorizes physical and virtual hardware functions:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceClass {
    Cpu,
    Memory,
    Block,
    Network,
    Gpu,
    Pci,
    Usb,
    System,
    Other,
}
```

### 2.2 `DeviceBus` (Enum)
Specifies the hardware interconnect bus:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceBus {
    Pci,
    Usb,
    Platform,
    Scsi,
    Virtio,
    System,
    Unknown,
}
```

### 2.3 `HardwareDevice` (Struct)
Represents an individual hardware component:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareDevice {
    pub id: String,
    pub name: String,
    pub class: DeviceClass,
    pub bus: DeviceBus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sysfs_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_path: Option<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}
```

### 2.4 `HardwareInventory` (Aggregate Struct)
Full host hardware manifest:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareInventory {
    pub timestamp: String,
    pub hostname: String,
    pub architecture: String,
    pub kernel_version: String,
    pub devices: Vec<HardwareDevice>,
    pub summary: BTreeMap<String, usize>,
}
```

---

## 3. Invariants & Validation Contracts (HD1..HD5)

| Code | Invariant Name | Mathematical Formulation | Failure Mode |
| :--- | :--- | :--- | :--- |
| **HD1** | Unique Identifiers | $\forall i \neq j, \text{devices}[i].\text{id} \neq \text{devices}[j].\text{id} \land \text{id} \neq \text{""}$ | Returns `Err("duplicate or empty device id")` |
| **HD2** | Hexadecimal Identifiers | $\forall d \in \text{devices}, d.\text{vendor\_id} \in [0-9a-fA-F]{4} \land d.\text{device\_id} \in [0-9a-fA-F]{4}$ | Returns `Err("invalid hex format for vendor_id/device_id")` |
| **HD3** | Summary Parity | $\forall c \in \text{DeviceClass}, \text{summary}[c] = \sum_{d \in \text{devices}} [d.\text{class} == c]$ | Returns `Err("summary count mismatch for class")` |
| **HD4** | Path Sanitization | $\forall d \in \text{devices}, \text{no control characters or traversal in sysfs\_path or dev\_path}$ | Returns `Err("unsafe path format in device")` |
| **HD5** | Deterministic JSON Roundtrip | $\text{from\_json}(\text{to\_json}(inv)) == inv$ | Returns `Err("serde roundtrip deserialization failed")` |

---

## 4. Helper Functions & Methods
1. `validate_device_id(id: &str) -> Result<(), String>`
2. `validate_hex_id(id: &str, field_name: &str) -> Result<(), String>`
3. `validate_path(path: &str, field_name: &str) -> Result<(), String>`
4. `HardwareInventory::new(hostname, arch, kernel) -> Self`
5. `HardwareInventory::add_device(&mut self, device: HardwareDevice) -> Result<(), String>`
6. `HardwareInventory::filter_by_class(&self, class: DeviceClass) -> Vec<&HardwareDevice>`
7. `HardwareInventory::to_json(&self) -> Result<String, String>`
8. `HardwareInventory::from_json(s: &str) -> Result<HardwareInventory, String>`

---

## 5. Acceptance Criteria Checklist
- [x] Full input, output, error cases, and data layout specified.
- [x] Invariants HD1 through HD5 defined with validation contracts.
- [x] Cross-platform support (Linux runtime vs Windows/macOS mock support).
- [x] Zero external unapproved dependencies added.
