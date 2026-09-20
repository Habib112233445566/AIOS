# T-01732: Hardware Detection — MCP/API Surface Specification

## Metadata
- **Task ID**: `T-01732`
- **Sub-Epic**: Sub-Epic 4: Hardware Detection MCP/API Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Mathematical & Protocol Invariants (HM1..HM5)

| Invariant | Name | Formal Definition & Guarantee |
| :--- | :--- | :--- |
| **HM1** | Schema Conformity | $\forall t \in \text{HardwareTools}, \text{tools/list}(t).\text{inputSchema}.\text{additionalProperties} = \text{false} \land \text{valid JSON Schema}$. |
| **HM2** | Envelope Uniformity | Every invocation yields JSON `{"ok": true, "tool": t, "data": ...}` on success, or `{"ok": false, "error": ...}` on failure. |
| **HM3** | Audit Trail Invariant | $\forall \text{call}(t), \exists e \in \text{AuditRing}: e.\text{tool} = t \land e.\text{hash\_chain\_valid}$. |
| **HM4** | Parameter Hygiene | All string arguments satisfy $\text{len} \le L_{\max}$ and contain zero ASCII control characters. |
| **HM5** | Hermetic Testability | Tools accept custom `sysfs_path` and `procfs_path` without requiring host `/sys` access or root privileges. |

---

## 2. Tool Specifications & Schemas

### 2.1 `aios.hardware.scan`
- **Description**: "Discover host hardware across all or filtered subsystems with summary statistics"
- **Properties**:
  - `classes`: `array` of strings (enum values: `cpu`, `gpu`, `block`, `network`, `usb`, `pci`, `system`, `memory`, `other`).
  - `include_attributes`: `boolean` (optional, default `true`).
  - `sysfs_path`: `string` (optional custom sysfs root).
  - `procfs_path`: `string` (optional custom procfs root).
  - `grant_id`: `string` (optional PEP authorization grant ID).
- **Return Data**: Full `HardwareInventory` JSON object.

### 2.2 `aios.hardware.list`
- **Description**: "List discovered hardware devices with optional class filtering"
- **Properties**:
  - `classes`: `array` of strings.
  - `sysfs_path`: `string`.
  - `procfs_path`: `string`.
  - `grant_id`: `string`.
- **Return Data**: `{"devices": Vec<HardwareDevice>, "count": usize}`.

### 2.3 `aios.hardware.get`
- **Description**: "Inspect full details, vendor/device IDs, paths, and attributes for a specific device ID"
- **Properties**:
  - `device_id`: `string` (**required**, trimmed, length $\le 256$, no control characters).
  - `sysfs_path`: `string`.
  - `procfs_path`: `string`.
  - `grant_id`: `string`.
- **Required**: `["device_id"]`
- **Return Data**: `{"device": HardwareDevice}`.

### 2.4 `aios.hardware.summary`
- **Description**: "Get device count summary aggregated by device classification"
- **Properties**:
  - `sysfs_path`: `string`.
  - `procfs_path`: `string`.
  - `grant_id`: `string`.
- **Return Data**: `{"summary": BTreeMap<String, usize>, "total": usize}`.

### 2.5 `aios.hardware.verify`
- **Description**: "Validate hardware inventory against mathematical invariants HD1..HD5 from live scan or serialized JSON file"
- **Properties**:
  - `file_path`: `string` (optional path to serialized inventory JSON, max 10MB).
  - `sysfs_path`: `string`.
  - `procfs_path`: `string`.
  - `grant_id`: `string`.
- **Return Data**: `{"valid": bool, "device_count": usize}`.

---

## 3. Error Model
- Invalid parameters (e.g. unknown class, path containing control characters, missing device ID): Returns `Result::Err` with descriptive message mapped to `{"ok": false, "error": "..."}`.
- Target device not found on `aios.hardware.get`: Returns `Result::Err("device '<id>' not found in hardware inventory")`.
- File verification failure on `aios.hardware.verify`: Returns `Result::Err("...")` detailing the violated invariants.
