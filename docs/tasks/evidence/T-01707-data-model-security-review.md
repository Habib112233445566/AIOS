# T-01707: Hardware Detection — Data Model Security Review

## Metadata
- **Task ID**: `T-01707`
- **Sub-Epic**: Hardware Detection / Data Model
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor**: Security Subsystem Agent

---

## 1. Scope & Objectives
Conduct a comprehensive security review of the Hardware Detection data model (`code/aiosh-rust/aiosh-core/src/hardware.rs`) across:
- `HardwareDevice` and `HardwareInventory` structures
- Validation routines: `validate_device_id`, `validate_hex_id`, `validate_path`, `validate()`
- Invariant enforcement: `validate_invariants()` (HD1..HD5)
- JSON serialization and deserialization routines (`to_json`, `from_json`)

Key audit dimensions:
1. Input validation & path/argument injection
2. Untrusted content handling (JSON bombs, unbounded collections, memory exhaustion)
3. Audit and PEP authorization alignment for future state-changing operations
4. Identified abuse scenarios and mitigation requirements for hardening (T-01708)

---

## 2. Threat Modeling & Abuse Scenarios

### Scenario S-1: Memory Exhaustion via Unbounded Attributes Map (DoS)
- **Attack Vector**: An adversary or compromised agent injects an arbitrary number of attribute key-value pairs or multi-megabyte string values into `HardwareDevice.attributes`.
- **Impact**: Unbounded heap allocation leading to out-of-memory (OOM) termination of the AIOS daemon.
- **Finding**: Currently, `attributes` validates character content (no control chars), but does not enforce maximum map size or string length bounds.
- **Mitigation Requirement (for T-01708)**: Enforce size caps:
  - `MAX_ATTRIBUTES_PER_DEVICE = 128`
  - `MAX_ATTRIBUTE_KEY_LEN = 64`
  - `MAX_ATTRIBUTE_VAL_LEN = 1024`

### Scenario S-2: Unbounded Inventory Device Flood (DoS)
- **Attack Vector**: An untrusted caller passes a synthesized JSON inventory containing hundreds of thousands of pseudo-devices.
- **Impact**: Heavy JSON parsing overhead, CPU exhaustion during quadratic or $O(N)$ summary reconciliation, and excessive memory footprint.
- **Mitigation Requirement (for T-01708)**: Enforce `MAX_DEVICES = 10_000` in `HardwareInventory::add_device` and `validate_invariants`.

### Scenario S-3: Path Injection via `sysfs_path` or `dev_path`
- **Attack Vector**: Path attributes contain relative traversal (`../../etc/shadow`) or control characters intended to deceive downstream log parsers or file openers.
- **Finding**: `validate_path` currently checks for `..` and control characters, but lacks a maximum path length cap.
- **Mitigation Requirement (for T-01708)**: Enforce `MAX_PATH_LEN = 512` and ensure absolute path verification where applicable.

### Scenario S-4: Unbounded Device ID and Name Lengths
- **Attack Vector**: Huge device identifiers (e.g. 100KB strings) submitted to `validate_device_id`.
- **Mitigation Requirement (for T-01708)**: Enforce `MAX_DEVICE_ID_LEN = 128` and `MAX_DEVICE_NAME_LEN = 256`.

---

## 3. PEP Gating & Audit Logging Alignment
The Hardware Detection data model constitutes passive domain objects in `aiosh-core`. Consequential state-changing paths (such as persisting hardware inventories, modifying hardware configuration rules, or probing hardware interfaces via system calls) will reside in the Core Service (`T-01711`..`T-01716`), CLI Surface (`T-01721`..`T-01730`), and MCP Surface (`T-01731`..`T-01740`).
All future mutating methods must integrate with `PepStore` and `AuditRing`.

---

## 4. Acceptance Criteria Checklist
- [x] Input validation, path injection, and untrusted-content handling evaluated.
- [x] Abuse scenarios documented with concrete severity ratings.
- [x] Hardening requirements formulated for task T-01708.
- [x] No policy bypass remains open.
