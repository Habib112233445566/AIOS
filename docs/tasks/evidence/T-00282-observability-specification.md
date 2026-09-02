# T-00282 — Release Packaging & Backup: Observability Specification

## Inputs, Outputs & Error Cases

### 1. Audit Ring Event Schema (Happy Path)
- **Input**: A successful `aios.release.generate` or `aios.backup.create` execution.
- **Output**: An `AuditRow` where `outcome = "success"`.
- **Detail**: The `outcome_detail` must contain the absolute path to the generated artifact and its computed SHA-256 hash.

### 2. Physical Subprocess Failure (Failure Path)
- **Input**: A failure during `physical_generate_iso` or `physical_create_zip` (e.g. `genisoimage` is missing or the disk is full).
- **Output**: An `AuditRow` where `outcome = "error"`.
- **Detail (The Enhancement)**: Instead of a generic `Physical generation failed`, the `outcome_detail` field must include the exact `stderr` capture or OS-level `io::Error` description emitted by the underlying process, enabling post-mortem debugging.

## Interfaces Reused
- **`aiosh-core/src/audit.rs`**: The existing `AuditRing` and `AuditRowInput` structs are used to persist this data.
- **`aiosh-core/src/release.rs`**: We will modify the `Result` mappings in this file to capture and bubble up the stringified IO errors directly into the `outcome_detail` formatting logic.

## AIOS-Specific Decisions
This design rejects the use of synchronous `stdout` streaming to the CLI/MCP during the long-running generation process. Instead, we adhere to the AIOS design principle where the final `AuditRow` serves as the sole source of truth for the system's state and outcome. Asynchronous job progress tracking is deferred to a future epic (Phase 2+).
