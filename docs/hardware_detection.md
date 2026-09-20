# AIOS Hardware Detection & Inventory Subsystem

## 1. Architectural Overview

Hardware Detection is the host introspection and device discovery subsystem of AIOS (`Phase 1 — Linux Base System & Bootable Target / Hardware Detection`, tasks `T-01701` through `T-01800`).

The subsystem catalogs host hardware across multiple physical and virtual buses:
- **Processor & Execution Cores**: CPU models, core/thread topology, architecture flags.
- **Physical Memory & Virtual Topology**: RAM modules, NUMA nodes, swap devices.
- **Storage Subsystems**: NVMe, SATA, SCSI, VirtIO block devices, partitions, rotational flags.
- **Network Interface Controllers**: Ethernet, 802.11 Wi-Fi, virtual bridges, MAC addresses.
- **Graphics & Accelerated Compute**: Discrete and integrated GPUs (NVIDIA, Intel, AMD).
- **Expansion Buses & Peripherals**: PCI/PCIe hierarchy, USB root hubs and attached peripherals.
- **System Platform & Firmware**: DMI / SMBIOS platform identification, chassis, BIOS release.

All device state is modeled in `aiosh-core::hardware` using strongly-typed, deterministic domain entities.

---

## 2. Domain Data Model (Sub-Epic 1)

### 2.1 Functional Device Classification (`DeviceClass`)
```rust
pub enum DeviceClass {
    Cpu,      // Processors, cores, execution threads
    Memory,   // RAM, swap, physical memory modules
    Block,    // Storage block devices (NVMe, SATA, SCSI, virtual disks)
    Network,  // Network interface controllers (NICs, Wi-Fi, virtual links)
    Gpu,      // Display and compute accelerators
    Pci,      // Generic PCI host bridges and non-specialized controllers
    Usb,      // USB host controllers, hubs, and peripherals
    System,   // Motherboard, DMI/SMBIOS, chassis, firmware
    Other,    // Fallback classification for unrecognized devices
}
```

### 2.2 Interconnect Bus (`DeviceBus`)
```rust
pub enum DeviceBus {
    Pci,      // PCI, PCI Express
    Usb,      // USB 1.1, 2.0, 3.x, USB4
    Platform, // Linux platform pseudo-bus
    Scsi,     // SCSI, SATA, SAS
    Virtio,   // VirtIO virtual bus
    System,   // Direct memory mapped / system board
    Unknown,  // Fallback bus identifier
}
```

### 2.3 Individual Device Entity (`HardwareDevice`)
```rust
pub struct HardwareDevice {
    pub id: String,                         // Unique deterministic identifier (e.g. "pci:0000:00:02.0")
    pub name: String,                       // Human-readable model name
    pub class: DeviceClass,                 // Functional classification
    pub bus: DeviceBus,                     // Physical/virtual interconnect bus
    pub vendor_id: Option<String>,          // 4-digit hex vendor code (e.g. "8086")
    pub device_id: Option<String>,          // 4-digit hex device code (e.g. "4680")
    pub vendor_name: Option<String>,        // Resolved vendor name (e.g. "Intel Corporation")
    pub device_name: Option<String>,        // Resolved device product name
    pub driver: Option<String>,             // Active Linux kernel driver (e.g. "i915", "nvme")
    pub sysfs_path: Option<String>,         // Canonical sysfs path (e.g. "/sys/bus/pci/devices/...")
    pub dev_path: Option<String>,           // Associated device node (e.g. "/dev/dri/card0")
    pub attributes: BTreeMap<String, String>, // Deterministic key-value attribute metadata
}
```

### 2.4 Aggregate Inventory (`HardwareInventory`)
```rust
pub struct HardwareInventory {
    pub timestamp: String,                  // ISO-8601 UTC manifest generation timestamp
    pub hostname: String,                   // Host computer name
    pub architecture: String,               // Machine architecture (e.g. "x86_64", "aarch64")
    pub kernel_version: String,             // Active Linux kernel release string
    pub devices: Vec<HardwareDevice>,       // Ordered list of discovered hardware devices
    pub summary: BTreeMap<String, usize>,   // Deterministic device counts per classification
}
```

---

## 3. Mathematical Invariants (HD1..HD5)

| Invariant | Name | Guarantee & Enforcement | Verification Test |
| :--- | :--- | :--- | :--- |
| **HD1** | Unique Identifiers | $\forall i \neq j, \text{devices}[i].\text{id} \neq \text{devices}[j].\text{id} \land \text{id} \neq \text{""} \land \text{len} \le 128$. | `test_hd1_duplicate_device_id_rejection` |
| **HD2** | Hexadecimal Identifiers | When present, $\text{vendor\_id}, \text{device\_id} \in [0-9a-fA-F]{4}$. | `test_hardware_device_validation_invalid_hex` |
| **HD3** | Summary Parity | $\forall c \in \text{DeviceClass}, \text{summary}[c] = \sum_{d \in \text{devices}} [d.\text{class} == c]$. | `test_hd3_summary_parity` |
| **HD4** | Path Sanitization | $\forall d \in \text{devices}$, paths must contain no control chars, no whitespace, no traversal (`..`), and $\text{len} \le 512$. | `test_hardware_device_validation_invalid_paths` |
| **HD5** | Lossless Serialization | $\text{from\_json}(\text{to\_json}(\text{inv})) == \text{inv}$. Deterministic key ordering via `BTreeMap`. | `test_hd5_json_roundtrip_and_deterministic_order` |

---

## 4. Hardening Bounds & Security Constraints

The data model enforces strict limits to prevent denial-of-service (DoS) and memory exhaustion:
- `MAX_DEVICES`: 10,000 devices per inventory manifest.
- `MAX_DEVICE_ID_LEN`: 128 characters.
- `MAX_DEVICE_NAME_LEN`: 256 characters.
- `MAX_ATTRIBUTES_PER_DEVICE`: 128 key-value pairs.
- `MAX_ATTRIBUTE_KEY_LEN`: 64 characters.
- `MAX_ATTRIBUTE_VAL_LEN`: 1,024 characters.
- `MAX_PATH_LEN`: 512 characters.
- `MAX_JSON_PAYLOAD_SIZE`: 10 MB (10,485,760 bytes).

---

## 5. Canonical Examples

### 5.1 Rust Builder API
```rust
use aiosh_core::hardware::{HardwareDevice, HardwareInventory, DeviceClass, DeviceBus};

let mut inv = HardwareInventory::new("aios-node-01", "x86_64", "6.6.13-aios-hardened");

let gpu = HardwareDevice::new("pci:0000:01:00.0", "NVIDIA GeForce RTX 4090", DeviceClass::Gpu, DeviceBus::Pci)
    .with_vendor("10de", Some("NVIDIA Corporation".into()))
    .with_device("2684", Some("AD102 [GeForce RTX 4090]".into()))
    .with_driver("nvidia")
    .with_paths(Some("/sys/bus/pci/devices/0000:01:00.0".into()), Some("/dev/dri/card1".into()))
    .with_attribute("vram_mb", "24576")
    .with_attribute("cuda_cores", "16384");

inv.add_device(gpu).expect("add gpu");
assert!(inv.validate_invariants().is_ok());
```

### 5.2 JSON Manifest
```json
{
  "timestamp": "2026-09-20T06:00:00Z",
  "hostname": "aios-node-01",
  "architecture": "x86_64",
  "kernel_version": "6.6.13-aios-hardened",
  "devices": [
    {
      "id": "pci:0000:01:00.0",
      "name": "NVIDIA GeForce RTX 4090",
      "class": "gpu",
      "bus": "pci",
      "vendor_id": "10de",
      "device_id": "2684",
      "vendor_name": "NVIDIA Corporation",
      "device_name": "AD102 [GeForce RTX 4090]",
      "driver": "nvidia",
      "sysfs_path": "/sys/bus/pci/devices/0000:01:00.0",
      "dev_path": "/dev/dri/card1",
      "attributes": {
        "cuda_cores": "16384",
        "vram_mb": "24576"
      }
    }
  ],
  "summary": {
    "gpu": 1
  }
}
```

---

## 6. Constraints & Known Limitations
- **Passive Data Model**: The data model defines representations and integrity contracts; active kernel sysfs scanning and device probing are handled by `HardwareService` (`T-01711`..`T-01716`).
- **Vendor ID Scope**: 4-hex digit vendor/device IDs primarily apply to PCI and USB devices; platform devices (e.g. system timers, ACPI devices) typically omit vendor/device hex IDs and identify by name/path.

---

## 7. Verification Evidence (Sub-Epic 1: Data Model)
- Research: `docs/tasks/evidence/T-01701-data-model-research.md`.
- Specification: `docs/tasks/evidence/T-01702-data-model-specification.md`.
- Scaffold: `docs/tasks/evidence/T-01703-data-model-scaffold.md`.
- Implementation: `docs/tasks/evidence/T-01704-data-model-implementation.md`.
- Unit Test: `docs/tasks/evidence/T-01705-data-model-unit-test.md`.
- Integration: `docs/tasks/evidence/T-01706-data-model-integration.md`.
- Security Review: `docs/tasks/evidence/T-01707-data-model-security-review.md`.
- Hardening: `docs/tasks/evidence/T-01708-data-model-hardening.md`.
- Documentation: `docs/tasks/evidence/T-01709-data-model-documentation.md`.
- Verification & Evidence: `docs/tasks/evidence/T-01710-data-model-verification-evidenc.md`.

---

## 8. Hardware Detection Core Service (Sub-Epic 2)

### 8.1 Discovery Engine (`HardwareService`)
The discovery engine lives in `aiosh_core::hardware_service::HardwareService` and orchestrates multi-subsystem host introspection:

```rust
use aiosh_core::{HardwareService, HardwareScanOptions, DeviceClass};

// Default targeting host /sys and /proc:
let service = HardwareService::new();

// Or targeting custom roots for hermetic mock testing:
let mock_service = HardwareService::with_roots("/tmp/mock_sys", "/tmp/mock_proc");

// Scan all subsystems:
let options = HardwareScanOptions::default();
let inventory = service.scan(&options)?;
```

### 8.2 Subsystem Probers
1. **PCI Subsystem (`probe_pci`)**:
   - Enumerates `/sys/bus/pci/devices/`.
   - Parses `vendor`, `device`, `class` (maps `0x03` $\to$ `Gpu`, `0x01` $\to$ `Block`, `0x02` $\to$ `Network`, else `Pci`).
   - Resolves active kernel driver via driver symlink.
2. **USB Subsystem (`probe_usb`)**:
   - Enumerates `/sys/bus/usb/devices/`.
   - Parses `idVendor`, `idProduct`, `manufacturer`, `product`, `speed`.
3. **Block Subsystem (`probe_block`)**:
   - Enumerates `/sys/class/block/`.
   - Extracts disk capacity (`size` sectors), rotational status (`queue/rotational`), and model (`device/model`).
4. **Network Subsystem (`probe_net`)**:
   - Enumerates `/sys/class/net/`.
   - Extracts MAC address (`address`), link status (`operstate`), and interface speed (`speed`).
5. **CPU Subsystem (`probe_cpu`)**:
   - Inspects `/proc/cpuinfo` and `/sys/devices/system/cpu/`.
   - Computes physical and logical core counts and processor model names.
6. **System / DMI Subsystem (`probe_system`)**:
   - Inspects `/sys/class/dmi/id/`.
   - Extracts motherboard vendor (`sys_vendor`), model (`product_name`), BIOS version (`bios_version`), and chassis type (`chassis_type`).

### 8.3 Invariants Matrix (HS1..HS5)
| Invariant | Name | Guarantee & Enforcement | Verification Test |
| :--- | :--- | :--- | :--- |
| **HS1** | Graceful Degradation | Missing `/sys` or `/proc` yields an empty, valid inventory without failing or panicking. | `test_hardware_service_empty_sysfs_resilience` |
| **HS2** | Accurate Classification | PCI class codes properly mapped to target `DeviceClass` enums. | `test_hardware_service_class_filtering` |
| **HS3** | Deterministic Ordering | Devices in generated inventories sorted lexicographically by device ID. | `test_hardware_service_crate_root_integration` |
| **HS4** | Input Sanitization | All hex identifiers normalized to 4-digit lowercase hex; path traversal sequences rejected. | `test_hs4_sanitization_and_normalization` |
| **HS5** | Inventory Integrity | Every generated inventory satisfies all mathematical invariants HD1..HD5. | `validate_hardware_inventory` assertion |

### 8.4 Verification Evidence (Sub-Epic 2: Core Service)
- Research: `docs/tasks/evidence/T-01711-core-service-research.md`.
- Specification: `docs/tasks/evidence/T-01712-core-service-specification.md`.
- Scaffold: `docs/tasks/evidence/T-01713-core-service-scaffold.md`.
- Implementation: `docs/tasks/evidence/T-01714-core-service-implementation.md`.
- Unit Test: `docs/tasks/evidence/T-01715-core-service-unit-test.md`.
- Integration: `docs/tasks/evidence/T-01716-core-service-integration.md`.
- Security Review: `docs/tasks/evidence/T-01717-core-service-security-review.md`.
- Hardening: `docs/tasks/evidence/T-01718-core-service-hardening.md`.
- Documentation: `docs/tasks/evidence/T-01719-core-service-documentation.md`.
- Verification & Evidence: `docs/tasks/evidence/T-01720-core-service-verification-evidenc.md`.

---

## 9. Hardware Detection CLI Surface (Sub-Epic 3)

### 9.1 Command Overview & Syntax
The CLI surface is accessed via `aiosh hw` (or `aiosh hardware`) in the Rust `aiosh-cli` binary:

```text
aiosh hw <scan|list|show|summary|verify> [OPTIONS]
```

### 9.2 Subcommands
| Subcommand | Description | Arguments & Flags |
| :--- | :--- | :--- |
| `scan` | Discovers host hardware across all or filtered subsystems | `--class <name>`, `--no-attrs`, `--sysfs <path>`, `--procfs <path>`, `--json` |
| `list` | Displays tabular overview of discovered hardware devices | `--class <name>`, `--sysfs <path>`, `--procfs <path>`, `--json` |
| `show <id>` | Displays full attributes and metadata for a single device ID | `<device_id>`, `--sysfs <path>`, `--procfs <path>`, `--json` |
| `summary` | Displays device counts aggregated by functional class | `--class <name>`, `--sysfs <path>`, `--procfs <path>`, `--json` |
| `verify` | Validates inventory integrity against invariants HD1..HD5 | `--file <path>`, `--sysfs <path>`, `--procfs <path>`, `--json` |

### 9.3 Options & Flags
- `--class <name>`: Filter discovery to a single device class (`cpu`, `gpu`, `block`, `network`, `usb`, `pci`, `system`, `memory`, `other`).
- `--no-attrs`: Exclude detailed device attributes from discovery to optimize payload size.
- `--sysfs <path>`: Specify a custom sysfs root (enables hermetic testing or alternate root mounts).
- `--procfs <path>`: Specify a custom procfs root.
- `--file <path>`: Specify a serialized inventory JSON file for offline invariant verification.
- `--json`: Format all output in the standardized AIOS JSON envelope (`{"code": 0, "data": ..., "error": null}`).

### 9.4 Exit Code Conventions
- `0`: Success — command executed and completed successfully.
- `1`: Operation Failure — device not found, inventory invariant failure, or I/O failure during scan.
- `2`: Parameter / Invocation Error — missing required device ID, unknown subcommand, invalid device class, path exceeding 1024 chars, or presence of control characters.

### 9.5 Security & Audit Invariants (CS1..CS4)
- **CS1 (Terminal Injection Prevention)**: All human-readable output strings are sanitized via `sanitize_terminal` to strip ANSI escape codes.
- **CS2 (Device ID Hygiene)**: Device IDs on `show` are trimmed, non-empty, capped at 256 characters, and checked for control characters.
- **CS3 (Buffer Limits)**: Input JSON files for `--file` verification are strictly capped at 10 MB.
- **CS4 (Audit Trail Logging)**: Every invocation, whether successful or rejected during parameter validation, is recorded in the SQLite WAL audit ring with category `hardware`.

### 9.6 Sub-Epic 3 Verification Evidence
- Research: `docs/tasks/evidence/T-01721-cli-surface-research.md`.
- Specification: `docs/tasks/evidence/T-01722-cli-surface-specification.md`.
- Scaffold: `docs/tasks/evidence/T-01723-cli-surface-scaffold.md`.
- Implementation: `docs/tasks/evidence/T-01724-cli-surface-implementation.md`.
- Unit Test: `docs/tasks/evidence/T-01725-cli-surface-unit-test.md`.
- Integration: `docs/tasks/evidence/T-01726-cli-surface-integration.md`.
- Security Review: `docs/tasks/evidence/T-01727-cli-surface-security-review.md`.
- Hardening: `docs/tasks/evidence/T-01728-cli-surface-hardening.md`.
- Documentation: `docs/tasks/evidence/T-01729-cli-surface-documentation.md`.
- Verification & Evidence: `docs/tasks/evidence/T-01730-cli-surface-verification-evidenc.md`.

---

## 10. Hardware Detection MCP & Agent Surface (Sub-Epic 4)

### 10.1 Overview & Architecture
The Model Context Protocol (MCP) surface is exposed via the `aiosh-mcp` binary communicating over JSON-RPC 2.0 stdio. It provides programmatic host hardware discovery, inspection, and verification capabilities to AI agents, orchestrators, and automated reasoning tools.

All tool invocations route through `dispatch::recorded_call`, enforcing Policy Enforcement Point (PEP) evaluation and writing SHA-256 hash-chained records to the SQLite WAL audit ring.

### 10.2 Tool Catalog & Schemas
| Tool Name | Purpose | Key Arguments | Return Data |
| :--- | :--- | :--- | :--- |
| `aios.hardware.scan` | Comprehensive host discovery across subsystems | `classes` (list), `include_attributes` (bool), `sysfs_path`, `procfs_path` | Complete `HardwareInventory` JSON |
| `aios.hardware.list` | Filtered list of discovered hardware devices | `classes` (list), `sysfs_path`, `procfs_path` | `{"devices": [...], "count": usize}` |
| `aios.hardware.get` | Detailed inspection of a specific device | `device_id` (string, required), `sysfs_path`, `procfs_path` | `{"device": <HardwareDevice>}` |
| `aios.hardware.summary` | Aggregate device counts per classification | `sysfs_path`, `procfs_path` | `{"summary": {...}, "total": usize}` |
| `aios.hardware.verify` | Invariant verification (live or file-based) | `file_path` (string), `sysfs_path`, `procfs_path` | `{"valid": bool, "device_count": usize}` |

### 10.3 Protocol Invariants (HM1..HM5)
- **HM1 (Schema Conformity)**: Every hardware tool sets `"additionalProperties": false` in its JSON Schema in `tools/list`.
- **HM2 (Envelope Uniformity)**: Every tool returns `{"ok": true, "tool": "<name>", "data": ...}` on success, or `{"ok": false, "error": "<msg>"}` on error.
- **HM3 (Audit Trail Invariant)**: Every call (including early validation rejections) is recorded in the SQLite WAL audit ring.
- **HM4 (Parameter Hygiene)**: String arguments enforce strict bounds: paths $\le 1024$ chars, device IDs $\le 256$ chars, control characters rejected, offline verify files $\le 10$ MB.
- **HM5 (Hermetic Testability)**: Tools accept `sysfs_path` and `procfs_path` overrides, enabling isolated execution without host `/sys` access or root privileges.

### 10.4 Sub-Epic 4 Verification Evidence
- Research: `docs/tasks/evidence/T-01731-mcp-api-research.md`.
- Specification: `docs/tasks/evidence/T-01732-mcp-api-specification.md`.
- Scaffold: `docs/tasks/evidence/T-01733-mcp-api-scaffold.md`.
- Implementation: `docs/tasks/evidence/T-01734-mcp-api-implementation.md`.
- Unit Test: `docs/tasks/evidence/T-01735-mcp-api-unit-test.md`.
- Integration: `docs/tasks/evidence/T-01736-mcp-api-integration.md`.
- Security Review: `docs/tasks/evidence/T-01737-mcp-api-security-review.md`.
- Hardening: `docs/tasks/evidence/T-01738-mcp-api-hardening.md`.
- Documentation: `docs/tasks/evidence/T-01739-mcp-api-documentation.md`.
- Verification & Evidence: `docs/tasks/evidence/T-01740-mcp-api-verification-evidenc.md`.

---

## 11. Hardware Detection Configuration Subsystem (Sub-Epic 5)

### 11.1 Overview & Architecture
The Hardware Detection configuration subsystem is implemented in `aiosh-core::hardware_config` and managed via the `HardwareConfig` structure. It provides persistent storage path configuration, scan resource limits, device filtering controls, and environment variable overrides for host introspection services.

### 11.2 Configuration Contract (`HardwareConfig`)
```rust
pub struct HardwareConfig {
    pub default_store_path: PathBuf,            // Path to saved inventory JSON
    pub sysfs_path: PathBuf,                    // Root to sysfs (defaults to /sys)
    pub procfs_path: PathBuf,                   // Root to procfs (defaults to /proc)
    pub enabled_classes: Option<Vec<DeviceClass>>, // Whitelist of device classes (None = all)
    pub include_attributes: bool,               // Whether to collect extended device attributes
    pub max_devices: usize,                     // Resource cap: maximum devices per scan
    pub max_payload_bytes: u64,                 // Resource cap: maximum inventory JSON bytes
    pub scan_timeout_secs: u64,                 // Timeout in seconds for discovery scans
}
```

### 11.3 Configuration Invariants (HCFG1..HCFG5)
- **HCFG1 (Path Hygiene & Traversal Prevention)**: All paths (`default_store_path`, `sysfs_path`, `procfs_path`) must be non-empty, UTF-8 valid, $\le 1024$ characters, free of control characters/NUL bytes, and must not contain parent directory traversal components (`..`).
- **HCFG2 (Class Filtering & Uniqueness)**: When `enabled_classes` is configured, it must contain $\le 9$ items, valid `DeviceClass` variants, and zero duplicates.
- **HCFG3 (Resource Bounds)**: `max_devices` is bounded ($1 \le n \le 50,000$); `max_payload_bytes` is bounded ($1024 \le n \le 104,857,600$).
- **HCFG4 (Timeout Bounds)**: `scan_timeout_secs` is bounded ($1 \le n \le 300$).
- **HCFG5 (Lossless Serialization & Safe Fallback)**:
  - `load_from_path()` safely falls back to default configuration when the target file does not exist.
  - `save_to_path()` executes atomic writes via a sibling temporary file (`.<name>.tmp.<pid>`) and atomic rename.
  - `from_env()` loads environment variable overrides and enforces post-validation, safely reverting to defaults if an invalid state is detected.

### 11.4 Environment Variable Mappings
| Variable Name | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `AIOSH_HARDWARE_CONFIG` | Path | None | Path to custom JSON configuration file |
| `AIOSH_HARDWARE_SYSFS` | Path | `/sys` | Overrides root sysfs introspection path |
| `AIOSH_HARDWARE_PROCFS` | Path | `/proc` | Overrides root procfs introspection path |
| `AIOSH_HARDWARE_STORE` | Path | `.aios/hardware_inventory.json` | Overrides inventory persistence path |
| `AIOSH_HARDWARE_INCLUDE_ATTRS` | Bool | `true` | `"1"`, `"true"`, `"0"`, `"false"` |
| `AIOSH_HARDWARE_TIMEOUT_SECS` | Integer | `30` | Scan timeout in seconds ($1 \le n \le 300$) |

### 11.5 Sub-Epic 5 Verification Evidence
- Research: `docs/tasks/evidence/T-01741-configuration-research.md`.
- Specification: `docs/tasks/evidence/T-01742-configuration-specification.md`.
- Scaffold: `docs/tasks/evidence/T-01743-configuration-scaffold.md`.
- Implementation: `docs/tasks/evidence/T-01744-configuration-implementation.md`.
- Unit Test: `docs/tasks/evidence/T-01745-configuration-unit-test.md`.
- Integration: `docs/tasks/evidence/T-01746-configuration-integration.md`.
- Security Review: `docs/tasks/evidence/T-01747-configuration-security-review.md`.
- Hardening: `docs/tasks/evidence/T-01748-configuration-hardening.md`.
- Documentation: `docs/tasks/evidence/T-01749-configuration-documentation.md`.
- Verification & Evidence: `docs/tasks/evidence/T-01750-configuration-verification-evidenc.md`.




