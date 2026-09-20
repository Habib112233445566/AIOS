# T-01712: Hardware Detection — Core Service Specification

## Metadata
- **Task ID**: `T-01712`
- **Sub-Epic**: Hardware Detection / Core Service
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Specification Engineer**: Antigravity Autonomous Agent

---

## 1. Scope & System Architecture
The Core Service (`code/aiosh-rust/aiosh-core/src/hardware_service.rs`) executes discovery scans against the Linux kernel sysfs and procfs hierarchies, building strongly-typed, validated `HardwareInventory` instances.

---

## 2. API Contracts & Types

### 2.1 `HardwareScanOptions`
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareScanOptions {
    /// Optional filter to restrict scanning to specific device classes.
    pub classes: Option<Vec<DeviceClass>>,
    /// Whether to collect extended device attributes (e.g. VRAM, cores, MAC).
    pub include_attributes: bool,
}

impl Default for HardwareScanOptions {
    fn default() -> Self {
        HardwareScanOptions {
            classes: None,
            include_attributes: true,
        }
    }
}
```

### 2.2 `HardwareService`
```rust
pub struct HardwareService {
    sysfs_root: PathBuf,
    procfs_root: PathBuf,
    cached_inventory: std::sync::RwLock<Option<HardwareInventory>>,
}

impl HardwareService {
    /// Initializes a HardwareService targeting the host Linux filesystem.
    pub fn new() -> Self;

    /// Initializes a HardwareService targeting custom root directories (for mock testing).
    pub fn with_roots(sysfs_root: impl Into<PathBuf>, procfs_root: impl Into<PathBuf>) -> Self;

    /// Executes a full or filtered hardware discovery scan.
    pub fn scan(&self, options: &HardwareScanOptions) -> Result<HardwareInventory, String>;

    /// Returns a copy of the cached inventory if available.
    pub fn get_cached_inventory(&self) -> Option<HardwareInventory>;

    /// Clears the cached inventory.
    pub fn invalidate_cache(&self);
}
```

---

## 3. Prober Specifications

| Prober | Input Sysfs Path | Key Probed Attributes | Output `DeviceClass` |
| :--- | :--- | :--- | :--- |
| **PCI** | `<sysfs>/bus/pci/devices/*` | `vendor`, `device`, `class`, `driver`, `subsystem_vendor` | `Gpu`, `Block`, `Network`, `Pci` (based on PCI class prefix) |
| **USB** | `<sysfs>/bus/usb/devices/*` | `idVendor`, `idProduct`, `manufacturer`, `product`, `speed` | `Usb`, `Block`, `Network`, `Other` |
| **Block** | `<sysfs>/class/block/*` | `size`, `removable`, `queue/rotational`, `device/model` | `Block` |
| **Network** | `<sysfs>/class/net/*` | `address`, `operstate`, `speed`, `type` | `Network` |
| **CPU** | `<procfs>/cpuinfo` & `<sysfs>/devices/system/cpu/*` | `model name`, core directories `cpu[0-9]+`, topology | `Cpu` |
| **System** | `<sysfs>/class/dmi/id/*` | `sys_vendor`, `product_name`, `bios_version` | `System` |

---

## 4. Invariants & Guarantees (HS1..HS5)
- **HS1 (Bounded Directory Iteration)**: Each prober iterates at most 1,024 entries per directory to prevent unbounded loops on virtual filesystems.
- **HS2 (Fault-Tolerant Skipping)**: Missing attribute files or I/O errors on individual device entries do not abort the scan; they are handled with safe fallbacks.
- **HS3 (Deterministic Ordering)**: The resulting `HardwareInventory.devices` list is deterministically sorted by device `id`.
- **HS4 (Inventory Integrity)**: Every returned `HardwareInventory` satisfies `validate_invariants()` (HD1..HD5).
- **HS5 (Thread Safety)**: `HardwareService` is `Send + Sync`, and cache operations use `std::sync::RwLock`.

---

## 5. Acceptance Criteria Checklist
- [x] Inputs, outputs, options, and error cases specified.
- [x] Sub-probers specified for all supported bus and class types.
- [x] Invariants HS1..HS5 defined.
- [x] Spec reviewable without reading implementation.
