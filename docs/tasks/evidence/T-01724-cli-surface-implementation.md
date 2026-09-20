# T-01724: Hardware Detection — CLI Surface Implementation

## Metadata
- **Task ID**: `T-01724`
- **Sub-Epic**: Sub-Epic 3: Hardware Detection CLI Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Implementation Overview
Implemented the full operator command handler `fn cmd_hardware(args: &[String]) -> i32` in `code/aiosh-rust/aiosh-cli/src/main.rs`:

1. **Path Hygiene Enforcement**:
   - Validates `--sysfs`, `--procfs`, and `--file` paths:
     - Rejects paths exceeding 1,024 characters with exit code 2 and error code `PATH_TOO_LONG`.
     - Rejects paths containing non-printable control characters with exit code 2 and error code `PATH_CONTAINS_CONTROL_CHAR`.

2. **Subcommand Handlers**:
   - `scan`: Executes hardware discovery across subsystems. Emits audit row. Formats human-readable summary table or canonical JSON envelope.
   - `list`: Lists discovered devices in tabular format or JSON array with `--json`.
   - `show <device-id>`: Displays detailed device properties, vendor/device IDs, bus, driver, and attributes. Returns exit code 2 (`MISSING_DEVICE_ID`) if argument omitted; exit code 1 (`DEVICE_NOT_FOUND`) if device does not exist.
   - `summary`: Outputs categorical device breakdown and total count.
   - `verify`: Validates live hardware scan or JSON file specified by `--file` against invariants HD1..HD5 (`validate_hardware_inventory`). Rejects files exceeding 10MB limit.

3. **Classification & Filtering**:
   - `--class <name>`: Parses and filters by `DeviceClass` (`cpu`, `gpu`, `block`, `network`, `usb`, `pci`, `system`, `memory`, `other`). Rejects unrecognized classes with exit code 2 (`INVALID_DEVICE_CLASS`).
   - `--no-attrs` / `--no-attributes`: Strips attribute metadata from returned devices.

4. **Audit Ring Integration**:
   - Every subcommand invocation records an audit row via `classify_and_emit` into the local SQLite WAL ring.

---

## 2. Compilation Verification
Command:
```bash
cargo check -p aiosh-cli
```
Output:
```
Checking aiosh-cli v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-cli)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 13.50s
```
Status: **PASS (0 errors, 0 warnings)**.
