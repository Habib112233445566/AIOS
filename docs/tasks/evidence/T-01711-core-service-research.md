# T-01711: Hardware Detection — Core Service Research

## Metadata
- **Task ID**: `T-01711`
- **Sub-Epic**: Hardware Detection / Core Service (Sub-Epic 2 of 10)
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection (`T-01701` .. `T-01800`)
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Research Engineer**: Antigravity Autonomous Agent

---

## 1. Scope & System Role
The Hardware Detection Core Service (`HardwareService`) is the active discovery and introspection engine that translates raw Linux kernel interfaces (`/sys`, `/proc`, DMI/SMBIOS) into validated `HardwareInventory` manifests.

---

## 2. Kernel Introspection Points & Authoritative Specifications

### 2.1 PCI Bus Introspection (`/sys/bus/pci/devices/*`)
- **Structure**: Each PCI device has a directory `/sys/bus/pci/devices/<domain>:<bus>:<slot>.<func>`.
- **Key Attributes**:
  - `vendor`: `0x8086\n` (Intel), `0x10de\n` (NVIDIA) — hex string prefixed by `0x`.
  - `device`: `0x4680\n` — 4-digit hex string.
  - `class`: `0x030000\n` — 24-bit integer representing PCI base class, sub-class, programming interface:
    - `0x01....` -> Mass Storage -> `DeviceClass::Block`
    - `0x02....` -> Network Controller -> `DeviceClass::Network`
    - `0x03....` -> Display Controller -> `DeviceClass::Gpu`
    - `0x06....` -> Bridge Device -> `DeviceClass::Pci`
  - `driver`: Symlink pointing to driver (e.g. `../../../../bus/pci/drivers/i915`).
  - `subsystem_vendor`, `subsystem_device`.

### 2.2 USB Bus Introspection (`/sys/bus/usb/devices/*`)
- **Structure**: USB devices registered with bus-port notation (e.g. `1-1`, `2-1.2`).
- **Key Attributes**:
  - `idVendor`: `046d\n`
  - `idProduct`: `c52b\n`
  - `manufacturer`: Human-readable vendor string.
  - `product`: Human-readable device string.
  - `speed`: Link bitrate in Mbps.

### 2.3 Storage Block Devices (`/sys/class/block/*`)
- **Structure**: Block devices (`sda`, `nvme0n1`, `vda`).
- **Key Attributes**:
  - `size`: 512-byte sector count.
  - `queue/rotational`: `0` for SSD/NVMe, `1` for mechanical HDD.
  - `removable`: `1` for USB flash/optical, `0` for fixed disks.
  - `device/model`, `device/vendor`.

### 2.4 Network Devices (`/sys/class/net/*`)
- **Structure**: Network interfaces (`eth0`, `enp0s31f6`, `wlan0`, `lo`).
- **Key Attributes**:
  - `address`: Link layer MAC address (`xx:xx:xx:xx:xx:xx`).
  - `operstate`: `up`, `down`, `unknown`.
  - `speed`: Link speed in Mbps.

### 2.5 CPU Topology (`/proc/cpuinfo` & `/sys/devices/system/cpu/*`)
- **Key Attributes**: Core counts, model name, hyperthreading siblings, vendor ID.

### 2.6 DMI / SMBIOS Platform Firmware (`/sys/class/dmi/id/*`)
- **Key Attributes**: `sys_vendor`, `product_name`, `bios_version`, `chassis_type`.

---

## 3. Fact vs. Assumption Matrix

| Category | Fact | Assumption |
| :--- | :--- | :--- |
| **Sysfs Path Root** | On Linux, sysfs mounts at `/sys`. | A mock root path parameter (`sysfs_root: Option<PathBuf>`) allows testing the engine on Windows and macOS using temporary directories. |
| **PCI / USB ID Stripping** | Sysfs PCI vendor files include `0x` prefixes (`0x8086`), while USB sysfs files omit `0x` (`8086`). | The core service must normalize both formats to 4-digit lowercase hex strings. |
| **Device Disconnects** | Devices can be dynamically unplugged or sysfs files unmapped concurrently during a scan. | Probers must treat missing attributes as non-fatal, skip or record defaults, and never crash. |
| **Performance Overhead** | Reading thousands of small pseudo-files in `/sys` can cause context switches. | `HardwareService` should maintain an in-memory cached inventory with time-to-live (TTL) to avoid redundant I/O churn. |

---

## 4. Key Architectural Decisions

1. **Configurable Sysfs Root**:
   - `HardwareService::new()` defaults to system `/sys`, `/proc`, etc.
   - `HardwareService::with_root(root: PathBuf)` operates over a designated directory, enabling 100% mock sysfs unit and integration testing without root privileges or a Linux host.
2. **Modular Prober Architecture**:
   - Dedicated probers: `probe_pci()`, `probe_usb()`, `probe_block()`, `probe_net()`, `probe_cpu()`, `probe_system()`.
   - Each prober operates independently and reports discovered `HardwareDevice` instances.
3. **Deterministic Ordering**:
   - All discovered devices are sorted by `id` prior to inventory creation.

---

## 5. Acceptance Criteria Checklist
- [x] Authoritative Linux kernel sysfs specifications cited.
- [x] Facts separated from assumptions.
- [x] Cross-platform mock sysfs architecture designed.
- [x] No source code changed during research.
