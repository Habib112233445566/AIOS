# T-02298: Grant Lifecycle Recovery & Validation Hardening

## Overview
This task documents the defensive hardening controls implemented across the PEP Grant Store Recovery and Invariant Validation subsystem to ensure operational resilience, fault isolation, and fail-safe behavior.

## Hardening Controls Implemented

### 1. Storage Size Limits & Bounded Memory Allocation
- **Ceiling**: `MAX_GRANT_SERVICE_STORE_SIZE = 10 * 1024 * 1024` (10 MiB).
- **Enforcement**: Prior to reading files into memory, `std::fs::metadata` is checked. Files exceeding the size limit immediately fail with a `Fatal` validation report or explicit capacity error.
- **Grant Capacity**: Service level caps grants at `MAX_GRANTS_IN_SERVICE = 5000`, preventing algorithmic slowdowns during validation graph traversals.

### 2. Path Traversal & File Extension Hardening
- **Path Validation**: `validate_grant_service_path` executes on all store file paths.
- **Constraints**:
  - Maximum path length: 1024 characters.
  - Prohibition of null bytes and ASCII control characters.
  - Strict rejection of `..` parent directory traversal components.
  - Mandatory `.json` file extension requirement.

### 3. Atomic Replacement & Crash Consistency
- **Safe Persistence**: Modified stores are never written directly to the target file path.
- **Atomic Swap**: Data is serialized, written to a process-unique temporary path (`<path>.tmp.<pid>.<timestamp>`), and moved via atomic `std::fs::rename`.
- **Cleanup**: In the event of a rename or writing failure, temporary files are cleanly unlinked to prevent lingering disk residue.

### 4. Non-Destructive Quarantine and Snapshot Architecture
- **Snapshots**: Point-in-time backup copies are created before any in-place mutation: `<path>.bak.<timestamp>`.
- **Corrupt File Isolation**: Severely corrupted files (e.g., malformed JSON or binary garbage) are preserved in a quarantine artifact (`<path>.quarantine.<timestamp>.json`) rather than truncated, allowing human operators to inspect and audit data salvage.

### 5. Uniform Error Envelopes & Fail-Closed Auditing
- All MCP endpoints return structured JSON response envelopes with explicit `ok: true/false` and typed error strings.
- Invocations are recorded to the SQLite WAL ring buffer before and after execution, ensuring full audit trail continuity even under transient failures.
