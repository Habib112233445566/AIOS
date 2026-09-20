# T-01722: Hardware Detection — CLI Surface Specification

## Metadata
- **Task ID**: `T-01722`
- **Sub-Epic**: Sub-Epic 3: Hardware Detection CLI Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. CLI Surface Specification (`aiosh hw` / `aiosh hardware`)

### 1.1 Command Syntax & Parameters
- **`aiosh hw scan`**:
  - Arguments:
    - `--class <class>`: Filter by device class (`cpu`, `gpu`, `block`, `network`, `usb`, `pci`, `system`).
    - `--no-attrs` / `--no-attributes`: Omit extended device attributes.
    - `--sysfs <path>`: Override sysfs root.
    - `--procfs <path>`: Override procfs root.
    - `--json`: Output standard JSON envelope.
  - Success (0): Returns full `HardwareInventory`.
- **`aiosh hw list`**:
  - Arguments: `--class <class>`, `--sysfs <path>`, `--procfs <path>`, `--json`.
  - Success (0): Returns `{ "devices": [...], "count": <usize> }`.
- **`aiosh hw show <device-id>`**:
  - Arguments: `<device-id>`, `--sysfs <path>`, `--procfs <path>`, `--json`.
  - Missing ID (2): Returns error `MISSING_DEVICE_ID`.
  - Device Not Found (1): Returns error `DEVICE_NOT_FOUND`.
  - Success (0): Returns `{ "device": <HardwareDevice> }`.
- **`aiosh hw summary`**:
  - Arguments: `--sysfs <path>`, `--procfs <path>`, `--json`.
  - Success (0): Returns `{ "summary": { "<class>": <count> }, "total": <usize> }`.
- **`aiosh hw verify`**:
  - Arguments: `[--file <path>]`, `--sysfs <path>`, `--procfs <path>`, `--json`.
  - Validation Failure (1): Returns error `VALIDATION_FAILED`.
  - Success (0): Returns `{ "valid": true, "device_count": <usize> }`.

---

## 2. Invariants Matrix (HC1..HC5)

| Invariant | Name | Guarantee & Enforcement | Verification Test |
| :--- | :--- | :--- | :--- |
| **HC1** | JSON Envelope Conformity | Every `--json` response conforms to `{ "code": <int>, "data": <val>, "error": <err> }`. | `test_hardware_cli_smoke.py` |
| **HC2** | Audit Row Emission | Every CLI invocation records an audit event in the local SQLite WAL ring. | CLI execution audit logging |
| **HC3** | Deterministic Exit Codes | Exit code 0 on success; 1 on domain error (not found/invalid); 2 on syntax/usage/path error. | `test_hardware_cli_smoke.py` |
| **HC4** | Class & Attribute Options | `--class` accurately filters output; `--no-attrs` strips attributes. | CLI unit & smoke tests |
| **HC5** | Path Sanitization | Reject paths $> 1024$ chars (`PATH_TOO_LONG`) or containing control chars (`PATH_CONTAINS_CONTROL_CHAR`). | `test_hw_path_hygiene` |

---

## 3. Acceptance Criteria Checklist
- [x] Command signatures and flags specified.
- [x] Exit codes 0, 1, 2 specified for all conditions.
- [x] Invariants HC1..HC5 formalized.
