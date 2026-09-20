# T-01701: Hardware Detection — Data Model Research

## Metadata
- **Task ID**: `T-01701`
- **Sub-Epic**: Hardware Detection / Data Model (Sub-Epic 1 of 10)
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection (`T-01701` .. `T-01800`)
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Research Engineer**: Antigravity Autonomous Agent

---

## 1. Problem Statement & Scope
Hardware Detection forms the base introspection layer for AIOS. To operate autonomously across bare-metal workstations, headless cloud VMs, and edge appliances, AIOS must accurately discover, categorize, and inventory host hardware components:
- Processors & Cores (CPU)
- Physical Memory (RAM & Swap)
- PCI/PCIe Expansion Devices & Buses
- USB Peripherals & Controllers
- Storage Block Devices (NVMe, SATA, SCSI, VirtIO)
- Network Interface Controllers (Ethernet, Wi-Fi, Virtual)
- Graphics Processing Units (Integrated & Discrete GPUs)
- System Platform & Firmware (DMI/SMBIOS)

This task researches the data model required to represent this inventory safely, deterministically, and losslessly across the Rust core (`aiosh-core`), CLI (`aiosh hw`), and MCP tool surface (`aios.hardware.*`).

---

## 2. Linux Introspection Architecture & Authoritative Sources

### 2.1 Authoritative Sources
1. **Linux Kernel Documentation**:
   - `Documentation/ABI/testing/sysfs-bus-pci`: Specification of PCI sysfs attributes (`vendor`, `device`, `class`, `driver`, `subsystem_vendor`, `subsystem_device`, `numa_node`).
   - `Documentation/ABI/testing/sysfs-bus-usb`: Specification of USB attributes (`idVendor`, `idProduct`, `speed`, `manufacturer`, `product`, `serial`, `busnum`, `devnum`).
   - `Documentation/ABI/testing/sysfs-block`: Block device properties (`size`, `removable`, `queue/rotational`, `ro`, `discard_alignment`).
   - `Documentation/ABI/testing/sysfs-class-net`: Network device attributes (`address`, `operstate`, `mtu`, `speed`, `duplex`, `type`).
2. **PCI Local Bus Specification / PCI-SIG**:
   - Class codes: 0x01 (Mass Storage), 0x02 (Network), 0x03 (Display), 0x04 (Multimedia), 0x06 (Bridge).
   - Vendor IDs: 16-bit hexadecimal identifiers registered by PCI-SIG (e.g. 0x8086 Intel, 0x10de NVIDIA, 0x1002 AMD).
3. **USB Implementers Forum (USB-IF)**:
   - Vendor IDs: 16-bit hex integers (e.g. 0x046d Logitech, 0x1d6b Linux Foundation root hub).
4. **DMI / SMBIOS Specification (DMTF DSP0134)**:
   - Structure table types accessible via `/sys/class/dmi/id/` (sys_vendor, product_name, bios_version).

---

## 3. Fact vs. Assumption Matrix

| Category | Fact | Assumption |
| :--- | :--- | :--- |
| **Sysfs Path Availability** | `/sys` is mounted as a virtual filesystem (`sysfs`) on modern Linux kernels (2.6+). | In test or non-Linux development environments (e.g. CI on Windows or macOS), mock sysfs file trees or fixtures must be supported. |
| **Identifier Formats** | PCI and USB vendor/device IDs are 4-digit lowercase hex strings (e.g. `8086`, `10de`). | Some virtual devices (VirtIO, Xen) may report pseudo IDs or strings without full PCI-SIG vendor tables. |
| **Device Classes** | Devices belong to standard functional classes (CPU, Memory, Block, Network, GPU, USB, PCI, System). | Subsystem classifications can be inferred accurately from sysfs `class` or PCI class codes. |
| **Serialization** | AIOS uses JSON with strict deterministic field ordering and ISO-8601 UTC timestamps. | All data models must support `serde::Serialize` and `serde::Deserialize` with no lossy conversions. |
| **Security & Auditing** | Introspecting sysfs is read-only for standard queries; no kernel mutations occur. | Reading `/sys` attributes does not require elevated root privileges except for raw physical PCI BAR mapping or sensitive DMI UUIDs. |

---

## 4. Unknowns & Technical Decisions

### Decision 1: Hardware Device Representation
- **Decision**: Define a strongly-typed enum `DeviceClass` (Cpu, Memory, Block, Network, Gpu, Pci, Usb, System, Other) and `DeviceBus` (Pci, Usb, Platform, Scsi, Virtio, Unknown).
- **Rationale**: Provides type safety while allowing generic queries and filtering.

### Decision 2: Hardware Device Entity Struct
- **Decision**: Implement `HardwareDevice` struct containing:
  - `id`: Unique deterministic identifier (e.g. `pci:0000:00:02.0`, `usb:1-1`, `cpu:0`, `block:nvme0n1`).
  - `name`: Human-readable description (e.g. "Intel Core i7-12700K", "NVIDIA GeForce RTX 4090").
  - `class`: `DeviceClass`.
  - `bus`: `DeviceBus`.
  - `vendor_id`: Optional 4-digit hex string (e.g. `8086`).
  - `device_id`: Optional 4-digit hex string (e.g. `4680`).
  - `vendor_name`: Optional string.
  - `device_name`: Optional string.
  - `driver`: Optional active driver name (e.g. `nvme`, `i915`, `e1000e`).
  - `sysfs_path`: Optional canonical path under `/sys`.
  - `dev_path`: Optional device node path (e.g. `/dev/nvme0n1`, `/dev/sda`).
  - `attributes`: Key-value map of string attributes (e.g. `size_bytes`, `cores`, `mac_address`, `speed`).

### Decision 3: Hardware Inventory Aggregate
- **Decision**: Implement `HardwareInventory` containing:
  - `timestamp`: ISO-8601 UTC creation time.
  - `hostname`: Host system name.
  - `architecture`: Machine architecture (e.g. `x86_64`, `aarch64`).
  - `kernel_version`: Active kernel release.
  - `devices`: `Vec<HardwareDevice>`.
  - `summary`: Invariant counts by class.
  - Validation method `validate_invariants(&self) -> Result<(), String>`.

---

## 5. Invariant Definitions (HD1..HD5)
- **HD1 (Unique Identifiers)**: Every device in `HardwareInventory.devices` must have a non-empty, unique `id`.
- **HD2 (Hex Identifier Format)**: When present, `vendor_id` and `device_id` must match 4-digit hexadecimal regex `^[0-9a-fA-F]{4}$`.
- **HD3 (Class Validation)**: Every device must specify a valid `DeviceClass`; counts in `summary` must exactly equal the sum of devices in each class.
- **HD4 (Path Sanitization)**: `sysfs_path` and `dev_path` must not contain null bytes, control characters, or path traversal sequences.
- **HD5 (Deterministic Serialization)**: Serializing a `HardwareInventory` to JSON and deserializing must be idempotent and roundtrip cleanly.

---

## 6. Acceptance Criteria Checklist
- [x] Authoritative Linux kernel sources and specifications referenced.
- [x] Facts separated from assumptions.
- [x] Data model architecture, struct layout, and invariant contracts HD1..HD5 defined.
- [x] No code modified in research task.
