# Task Evidence: T-01791 - Hardware Detection / Recovery & Validation: Research

## Metadata
- **Task ID:** `T-01791`
- **Sub-Epic:** Sub-Epic 10: Hardware Detection / Recovery & Validation
- **Component:** `aiosh-core::hardware_recovery`, `aiosh-core::hardware`
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Research Objectives & Scope
Investigate validation and recovery patterns for host hardware inventories and persistent store files:
1. Identify primary corruption modes, file-level errors, and data invariant violations in hardware inventories.
2. Formulate recovery strategies for corrupted JSON files, sysfs drift, and invalid device records.
3. Establish invariants `HVAL1..HVAL6` and architect `HardwareValidationReport` and `HardwareRecoveryReport`.
4. Align recovery mechanisms with existing production modules (`kernel_module_recovery.rs`, `distro_recovery.rs`).

## Failure Modes & Corruption Analysis
1. **File-Level Corruptions:**
   - Missing store file: Attempting to load from an uninitialized path.
   - Empty / truncated file: 0-byte file or incomplete JSON payload resulting from sudden power loss or process termination.
   - Malformed JSON: Syntax errors, unclosed brackets, or type mismatches.
   - Oversized payload: File exceeding `MAX_JSON_PAYLOAD_SIZE` (10 MB).
2. **Data Model Invariant Violations:**
   - Duplicate device IDs violating uniqueness constraints (HD1).
   - Missing required fields (`hostname`, `architecture`, `name`).
   - Invalid vendor ID or device ID formats (non-hex characters).
   - Summary map mismatch (HD3 summary out-of-sync with device list).
3. **Sysfs Path Drift:**
   - Stale devices: Inventory records devices with `sysfs_path` or `dev_path` that no longer exist on the host filesystem (hot-unplugged USB/PCI devices, unmounted disks).
   - Missing drivers: Hardware device exists but kernel driver was unbound or unloaded.

## Recovery Mechanisms
1. **Timestamped Non-Destructive Quarantine:**
   - Corrupted store files are never overwritten in-place or deleted.
   - When corruption is detected during recovery, the file is copied to `<filename>.bak.<rfc3339_timestamp>` for forensics before a clean inventory is regenerated.
2. **Surgical In-Memory Device Pruning:**
   - When only specific device entries violate validation rules (e.g., invalid attribute keys or bad hex IDs), the recovery engine prunes invalid devices while preserving valid entries.
   - The summary counts are automatically recomputed to restore summary parity.
3. **Automated Live Rescan Fallback:**
   - When an inventory is fatally corrupted, the system can fallback to triggering a fresh scan against `/sys` via `HardwareScanner`.

## Invariants Formulated (HVAL1..HVAL6)
- **HVAL1 (Device Accounting)**: `valid_devices + invalid_devices == total_devices`.
- **HVAL2 (Summary Reconciliation)**: `summary` matches the count of valid devices grouped by functional class.
- **HVAL3 (Health State Consistency)**: `healthy == (errors.is_empty() && invalid_devices == 0 && !drift_detected)`.
- **HVAL4 (Non-Destructive Quarantine)**: Damaged files on disk are quarantined to `.bak.<timestamp>` prior to replacement.
- **HVAL5 (Bounded Resource Limits)**: Maximum file size (10 MB) and device bounds (`MAX_DEVICES = 10,000`) enforced.
- **HVAL6 (Deterministic Canonical Output)**: Output reports are deterministically ordered and formatted.
