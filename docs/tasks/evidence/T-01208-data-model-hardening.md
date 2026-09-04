# T-01208: Package Management - Data Model: Hardening

## Metadata
- **Task ID:** `T-01208`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package`
- **Component:** Package Management Data Model Hardening
- **Status:** Complete

## 1. Hardening Defenses & Invariants

### 1. Payload & File Sizing Ceilings
- Enforced a hard 1 MiB (`1,048,576` bytes) ceiling on package specification files and inline JSON payloads in `aiosh package validate --spec <input>`.
- Any file exceeding 1 MiB is rejected with exit code 2 and structured error code `PAYLOAD_TOO_LARGE` before memory allocation.
- In-memory data model enforces caps:
  - `name`: max 128 bytes.
  - `version`: max 64 bytes.
  - `architecture`: max 64 bytes.
  - `description`: max 4,096 bytes.
  - `dependencies`: max 256 entries.
  - `installed_size_bytes`: max 100 GiB.
  - `actions`: max 256 entries per transaction.

### 2. Explicit Result Envelopes
- Never fails silently. Every failure mode produces an explicit JSON error envelope containing:
  - `code`: non-zero integer (`1` for file not found / operational error, `2` for validation failure).
  - `data`: `null`.
  - `error`: `{ "code": "VALIDATION_FAILED" | "PAYLOAD_TOO_LARGE" | "INVALID_JSON" | "INVALID_ARGUMENT", "message": "...", "errors": [...] }`.

### 3. Resource Cleanup & RAII Guarantees
- No persistent file descriptors or SQLite database locks held open across failure paths.
- Synchronous cleanup via Rust's RAII `Drop` implementation on all intermediate structures.

### 4. Honest Audit Trail
- All operations (both successful validations and rejections) emit an honest audit row to the SQLite WAL ring (`audit.db`) with SHA-256 hash chaining and classified constitutional rule flags.
