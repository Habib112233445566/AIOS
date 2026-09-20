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

---

## 12. Hardware Detection Automated Test Subsystem (Sub-Epic 6)

### 12.1 Overview & Architecture
The Automated Test Subsystem enables isolated, deterministic, and root-free testing of the entire hardware discovery pipeline across heterogeneous environments (bare metal, virtualized containers, Windows developer workstations, CI/CD runners).

Test fixtures are generated dynamically using `MockSysfsBuilder`, simulating PCI, USB, Storage Block, Network, CPU, and DMI/Platform topologies with full fault injection capabilities.

### 12.2 Test Harness API (`MockSysfsBuilder`)
```rust
pub struct MockSysfsBuilder {
    pub temp_dir: TempDir,
    pub sysfs_root: PathBuf,
    pub procfs_root: PathBuf,
}

impl MockSysfsBuilder {
    pub fn new() -> Self;
    pub fn add_pci(&mut self, slot: &str, vendor: &str, device: &str, class_code: &str, driver: Option<&str>) -> &mut Self;
    pub fn add_usb(&mut self, id: &str, vendor: &str, product: &str, manufacturer: &str, prod_name: &str) -> &mut Self;
    pub fn add_block(&mut self, name: &str, size_sectors: u64, rotational: bool, model: &str) -> &mut Self;
    pub fn add_net(&mut self, name: &str, mac: &str, speed: i32, operstate: &str) -> &mut Self;
    pub fn add_cpu(&mut self, cpu_id: usize, model: &str, mhz: f64) -> &mut Self;
    pub fn add_dmi(&mut self, vendor: &str, product: &str, version: &str) -> &mut Self;
    pub fn roots(&self) -> (&Path, &Path);
}
```

### 12.3 Automated Testing Invariants (AT1..AT5)
- **AT1 (Hermetic Isolation)**: All mock discovery executions run strictly within isolated temporary directories without accessing or reading the host `/sys` or `/proc` filesystems.
- **AT2 (Fault Injection Robustness)**: Probers must gracefully handle corrupted hex codes (`0xZZZZ`, `0x`), truncated files, missing optional attributes (`size`, `rotational`, `speed`), and symlinks without panicking.
- **AT3 (Deterministic Identification & Classification)**: Synthetic device fixtures must deterministically map to normalized device IDs (`pci:0000:00:02.0`, `usb:1-1`, `block:sda`, `net:eth0`, `cpu:0`, `system:dmi`) and accurate `DeviceClass` classifications.
- **AT4 (Invariant Compliance)**: Inventories generated through mock hierarchies must satisfy all domain model invariants (`HD1..HD5`) and deterministic device sorting (`HS3`).
- **AT5 (Scale & Traversal Bounds)**: Directory traversal must strictly respect `MAX_PROBE_ENTRIES = 1024`, ensuring scans complete well within the execution timeout budget.

### 12.4 Sub-Epic 6 Verification Evidence
- Research: `docs/tasks/evidence/T-01751-automated-tests-research.md`.
- Specification: `docs/tasks/evidence/T-01752-automated-tests-specification.md`.
- Scaffold: `docs/tasks/evidence/T-01753-automated-tests-scaffold.md`.
- Implementation: `docs/tasks/evidence/T-01754-automated-tests-implementation.md`.
- Unit Test: `docs/tasks/evidence/T-01755-automated-tests-unit-test.md`.
- Integration: `docs/tasks/evidence/T-01756-automated-tests-integration.md`.
- Security Review: `docs/tasks/evidence/T-01757-automated-tests-security-review.md`.
- Hardening: `docs/tasks/evidence/T-01758-automated-tests-hardening.md`.
- Documentation: `docs/tasks/evidence/T-01759-automated-tests-documentation.md`.
- Verification & Evidence: `docs/tasks/evidence/T-01760-automated-tests-verification-evidenc.md`.

---

## 13. Hardware Detection Security Policy Subsystem (Sub-Epic 7)

### 13.1 Overview & Architecture
The Hardware Detection Security Policy Subsystem (`aiosh-core::hardware_policy`) provides declarative gatekeeping, compliance reporting, and sensitive attribute sanitization over discovered hardware inventories.

It ensures that hostile or unauthorized hardware (e.g., untrusted USB devices, disallowed network interfaces, unknown buses) is blocked or reported, and that privacy-sensitive device telemetry (MAC addresses, UUIDs, serial numbers) is sanitized before entering audit records or user responses.

### 13.2 Policy Contract & Data Schema
```rust
pub enum HardwarePolicyMode {
    Enforcing,   // Fatal violations yield "deny" and strip violating devices
    Audit,       // Violations recorded in report, inventory remains intact
    Permissive,  // All devices allowed; violations not fatal
}

pub struct HardwareSecurityPolicy {
    pub mode: HardwarePolicyMode,
    pub disallowed_classes: Vec<DeviceClass>,
    pub disallowed_buses: Vec<DeviceBus>,
    pub prohibited_device_ids: Vec<String>,
    pub allowed_vendor_ids: Option<Vec<String>>,
    pub redact_sensitive_attributes: bool,
    pub max_devices_allowed: usize,
}

pub struct HardwarePolicyReport {
    pub verdict: String, // "allow", "deny", "audit"
    pub mode: HardwarePolicyMode,
    pub violations: Vec<HardwarePolicyViolation>,
    pub devices_evaluated: usize,
    pub devices_redacted: usize,
}
```

### 13.3 Security Invariants (HSEC1..HSEC5)
- **HSEC1 (Policy Precedence & Filtering)**: Prohibited device IDs and disallowed classes take precedence over allowlists. In `Enforcing` mode, fatal violations produce a `"deny"` verdict and filter offending devices out of the inventory.
- **HSEC2 (Attribute Redaction)**: Sensitive attributes matching keys `address`, `mac`, `serial`, `uuid`, `wwid` are automatically masked to `"<REDACTED>"` when `redact_sensitive_attributes` is true.
- **HSEC3 (Class & Bus Gatekeeping)**: Disallowed device classes and buses generate fatal violations (`HPOL-CLASS`, `HPOL-BUS`).
- **HSEC4 (Deterministic Reports)**: Policy evaluation is side-effect-free and deterministically sorted by `rule_id` and `device_id`.
- **HSEC5 (Fail-Safe Defaults & Hardening)**:
  - Default policy is `Enforcing` with redaction enabled and a 10,000 device ceiling.
  - Policy files are capped at `MAX_POLICY_FILE_BYTES = 1,048,576` (1 MB) to prevent OOM/DoS.
  - Policy file paths are validated against control characters, length $> 1024$, and traversal (`..`).
  - List entries are capped at $\le 10,000$ to prevent linear scan DoS.
  - Persistence executes via atomic sibling write (`.{name}.tmp.{pid}`) and rename.

### 13.4 CLI & MCP Usage Examples

#### Rust API Example
```rust
use aiosh_core::hardware_service::HardwareService;
use aiosh_core::hardware_policy::HardwareSecurityPolicy;

let service = HardwareService::new();
let policy = HardwareSecurityPolicy::default();

// Discovers hardware and applies security policy sanitization in one step
let (inventory, report) = service.scan_with_policy(None, true, None, None, &policy)?;
assert_eq!(report.verdict, "allow");
```

#### MCP Tool Example
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.hardware.scan",
    "arguments": {
      "classes": ["gpu", "network", "block"],
      "include_attributes": true
    }
  }
}
```

### 13.5 Sub-Epic 7 Verification Evidence
- Research: `docs/tasks/evidence/T-01761-security-policy-research.md`.
- Specification: `docs/tasks/evidence/T-01762-security-policy-specification.md`.
- Scaffold: `docs/tasks/evidence/T-01763-security-policy-scaffold.md`.
- Implementation: `docs/tasks/evidence/T-01764-security-policy-implementation.md`.
- Unit Test: `docs/tasks/evidence/T-01765-security-policy-unit-test.md`.
- Integration: `docs/tasks/evidence/T-01766-security-policy-integration.md`.
- Security Review: `docs/tasks/evidence/T-01767-security-policy-security-review.md`.
- Hardening: `docs/tasks/evidence/T-01768-security-policy-hardening.md`.
- Documentation: `docs/tasks/evidence/T-01769-security-policy-documentation.md`.
- Verification & Evidence: `docs/tasks/evidence/T-01770-security-policy-verification-evidenc.md`.

---

## 14. Hardware Detection Observability Subsystem (Sub-Epic 8)

### 14.1 Overview & Architecture
The Hardware Detection Observability Subsystem (`aiosh-core::hardware_observability`) produces structured telemetry reports, driver binding ratios, class and bus distributions, and security policy compliance summaries.

Designed for fleet monitoring and automated triage, reports provide high-level operational intelligence without disclosing sensitive low-level hardware attributes (MAC addresses, UUIDs, or serial numbers).

### 14.2 Observability Contract (`HardwareObservabilityReport`)
```rust
pub struct HardwareObservabilityReport {
    pub total_devices: usize,
    pub class_breakdown: BTreeMap<String, usize>,
    pub bus_breakdown: BTreeMap<String, usize>,
    pub driver_binding_count: usize,
    pub unbound_device_count: usize,
    pub driver_binding_rate: f64,
    pub total_attributes_count: usize,
    pub policy_compliant_count: usize,
    pub policy_violations_count: usize,
    pub prohibited_devices_found: Vec<String>,
    pub redacted_devices_count: usize,
    pub hostname: String,
    pub architecture: String,
    pub kernel_version: String,
    pub generated_at: String,
}
```

### 14.3 Observability Invariants (HO1..HO6)
- **HO1 (Class Breakdown Parity)**: The total device count equals the sum of device counts across all functional classes.
- **HO2 (Bus Breakdown Parity)**: The total device count equals the sum of device counts across all interconnect buses.
- **HO3 (Driver Binding Accounting)**: The total device count equals `driver_binding_count + unbound_device_count`.
- **HO4 (Driver Binding Rate Consistency)**: `driver_binding_rate` is a bounded float in $[0.0, 1.0]$, rounded to 4 decimals, with safe zero-division fallback (`0.0`) when no devices exist.
- **HO5 (Policy Compliance Telemetry)**: Reports compliant and violating device counts, with `prohibited_devices_found` capped at `MAX_PROHIBITED_DEVICES_REPORTED = 1,000` entries.
- **HO6 (Deterministic Canonical Serialization)**: All breakdown maps use `BTreeMap` to guarantee deterministic alphabetical key ordering in serialized JSON output. String fields are sanitized against control characters.

### 14.4 API Usage Example

```rust
use aiosh_core::hardware_service::{HardwareScanOptions, HardwareService};
use aiosh_core::hardware_policy::HardwareSecurityPolicy;

let service = HardwareService::new();
let options = HardwareScanOptions::default();
let policy = HardwareSecurityPolicy::default();

// Generate telemetry report with policy evaluation
let report = service.generate_observability_report(&options, Some(&policy))?;

println!("Total Devices: {}", report.total_devices);
println!("Driver Binding Rate: {:.2}%", report.driver_binding_rate * 100.0);
println!("Policy Compliant Devices: {}", report.policy_compliant_count);
```

### 14.5 Sub-Epic 8 Verification Evidence
- Research: `docs/tasks/evidence/T-01771-observability-research.md`.
- Specification: `docs/tasks/evidence/T-01772-observability-specification.md`.
- Scaffold: `docs/tasks/evidence/T-01773-observability-scaffold.md`.
- Implementation: `docs/tasks/evidence/T-01774-observability-implementation.md`.
- Unit Test: `docs/tasks/evidence/T-01775-observability-unit-test.md`.
- Integration: `docs/tasks/evidence/T-01776-observability-integration.md`.
- Security Review: `docs/tasks/evidence/T-01777-observability-security-review.md`.
- Hardening: `docs/tasks/evidence/T-01778-observability-hardening.md`.
- Documentation: `docs/tasks/evidence/T-01779-observability-documentation.md`.
- Verification & Evidence: `docs/tasks/evidence/T-01780-observability-verification-evidenc.md`.

---

## 15. Hardware Detection Documentation Subsystem (Sub-Epic 9)

### 15.1 Overview & Architecture
The Hardware Detection Documentation Subsystem (`aiosh-core::hardware_doc`) provides an embedded, self-contained, offline documentation index and knowledge repository for Linux hardware discovery, sysfs topologies, security gatekeeping policies, and fleet telemetry.

This subsystem provides zero-dependency offline help for operators, developers, and AI agents interacting via MCP tools or the CLI.

### 15.2 Documentation Invariants (HDOC1..HDOC6)
- **HDOC1 (Canonical Topic Set)**: Pre-registers 6 core topics covering the entire hardware detection domain:
  - `hw-sysfs-topology`: Linux Sysfs Hardware Topology and Probing (`discovery`).
  - `hw-security-policy`: Hardware Detection Security Policy and Gatekeeping (`security`).
  - `hw-observability-telemetry`: Hardware Observability and Fleet Telemetry (`observability`).
  - `hw-config-options`: Hardware Detection Configuration and Environment Overrides (`configuration`).
  - `hw-mcp-tools`: Hardware Detection MCP Tool Catalog (`architecture`).
  - `hw-troubleshooting`: Hardware Detection Troubleshooting and Diagnostics (`troubleshooting`).
- **HDOC2 (Defensive Lookup Bounds)**: Topic retrieval by ID enforces case-insensitivity, length limit (`MAX_TOPIC_ID_LEN = 64`), and strict ASCII alphanumeric/hyphen/underscore/dot character validation.
- **HDOC3 (Scored Relevance Search)**: Weighted query matching ranks results:
  - Exact Topic ID Match: +100 points.
  - Substring Topic ID Match: +40 points.
  - Tag Match: +50 points (exact) / +20 points (substring).
  - Title Match: +35 points.
  - Summary Match: +15 points.
  - Section Content Match: +10 points.
  - Snippets are dynamically extracted with UTF-8 character boundary safety.
  - Queries are capped at `MAX_DOC_QUERY_LEN = 256`, and search results are truncated to `MAX_DOC_SEARCH_RESULTS = 50`.
- **HDOC4 (Category Filtering)**: Supports filtering by `HardwareDocCategory` (`architecture`, `discovery`, `security`, `observability`, `configuration`, `troubleshooting`).
- **HDOC5 (Deterministic Markdown Rendering)**: `HardwareDocIndex::format_topic_markdown` formats topic metadata, structured sections, example commands, and authoritative references into standardized GitHub-flavored Markdown.
- **HDOC6 (Memory Footprint & Security)**: Completely in-memory, deterministic, zero-allocation search paths, with total footprint under 50 KB.

### 15.3 API Usage Example

```rust
use aiosh_core::hardware_doc::{HardwareDocCategory, HardwareDocIndex};

let index = HardwareDocIndex::new();

// Lookup topic by ID
if let Some(topic) = index.get_topic("hw-security-policy") {
    println!("{}", HardwareDocIndex::format_topic_markdown(topic));
}

// Search topics by query keyword
let results = index.search("sysfs", Some(HardwareDocCategory::Discovery));
for res in results {
    println!("Found {} (score: {}): {}", res.title, res.score, res.snippet);
}
```

### 15.4 Sub-Epic 9 Verification Evidence
- Research: `docs/tasks/evidence/T-01781-documentation-research.md`.
- Specification: `docs/tasks/evidence/T-01782-documentation-specification.md`.
- Scaffold: `docs/tasks/evidence/T-01783-documentation-scaffold.md`.
- Implementation: `docs/tasks/evidence/T-01784-documentation-implementation.md`.
- Unit Test: `docs/tasks/evidence/T-01785-documentation-unit-test.md`.
- Integration: `docs/tasks/evidence/T-01786-documentation-integration.md`.
- Security Review: `docs/tasks/evidence/T-01787-documentation-security-review.md`.
- Hardening: `docs/tasks/evidence/T-01788-documentation-hardening.md`.
- Documentation: `docs/tasks/evidence/T-01789-documentation-documentation.md`.
- Verification & Evidence: `docs/tasks/evidence/T-01790-documentation-verification-evidenc.md`.

---

## 16. Hardware Detection Recovery & Validation Subsystem (Sub-Epic 10)

### 16.1 Overview & Architecture
The Hardware Detection Recovery & Validation Subsystem (`aiosh-core::hardware_recovery`) provides non-destructive automated self-healing, corruption quarantine, sysfs path drift detection, and data invariant enforcement for host hardware inventories and persistent store files.

### 16.2 Invariants (HVAL1..HVAL6)
- **HVAL1 (Device Accounting Parity)**: The total device count equals valid devices plus invalid devices (`valid_devices + invalid_devices == total_devices`).
- **HVAL2 (Summary Reconciliation)**: Functional class summary counts are strictly verified against valid devices. The recovery engine automatically reconciles and recalculates summaries.
- **HVAL3 (Health State Consistency)**: An inventory is reported as `healthy` if and only if zero validation errors exist, zero invalid devices exist, no sysfs drift is detected, and no summary mismatches exist.
- **HVAL4 (Non-Destructive Quarantine)**: Corrupted, truncated, or unparseable store files are safely copied to a timestamped backup (`<filename>.bak.<timestamp>`) prior to re-initialization or surgical repair.
- **HVAL5 (Bounded Resource Limits & Path Hygiene)**: Store file sizes are strictly capped at `MAX_STORE_FILE_SIZE = 10 MB`. Store paths must be valid `.json` files free from parent directory traversal (`..`) or control characters.
- **HVAL6 (Sysfs Path Drift Detection)**: Supports live validation against host `/sys` paths. Devices referencing non-existent sysfs or `/dev` entries are flagged as drift (`drift_detected = true`).

### 16.3 API Usage Example

```rust
use std::path::Path;
use aiosh_core::hardware_service::HardwareService;

let service = HardwareService::new();
let store_path = Path::new("/etc/aios/hardware_inventory.json");

// 1. Inspect store health and sysfs drift
let val_report = service.validate_store(store_path, true)?;
if !val_report.healthy {
    println!("Store is unhealthy (drift: {}): {:?}", val_report.drift_detected, val_report.errors);

    // 2. Perform non-destructive automated recovery
    let rec_report = service.recover_store(store_path)?;
    println!("Store recovered: {}, backup at: {:?}", rec_report.recovered, rec_report.backup_path);
}
```

### 16.4 Sub-Epic 10 Verification Evidence
- Research: `docs/tasks/evidence/T-01791-recovery-research.md`.
- Specification: `docs/tasks/evidence/T-01792-recovery-specification.md`.
- Scaffold: `docs/tasks/evidence/T-01793-recovery-scaffold.md`.
- Implementation: `docs/tasks/evidence/T-01794-recovery-implementation.md`.
- Unit Test: `docs/tasks/evidence/T-01795-recovery-unit-test.md`.
- Integration: `docs/tasks/evidence/T-01796-recovery-integration.md`.
- Security Review: `docs/tasks/evidence/T-01797-recovery-validation-security-review.md`.
- Hardening: `docs/tasks/evidence/T-01798-recovery-validation-hardening.md`.
- Documentation: `docs/tasks/evidence/T-01799-recovery-validation-documentation.md`.
- Verification & Evidence: `docs/tasks/evidence/T-01800-recovery-validation-verification-evidenc.md`.









