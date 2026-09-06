# T-01308: Init & Service Supervision - Data Model: Hardening

## Metadata
- **Task ID:** `T-01308`
- **Subsystem:** `code/aiosh-rust/aiosh-core::service`
- **Component:** Init & Service Supervision Data Model Hardening
- **Status:** Complete

## 1. Hardening Defenses & Invariants

### 1. Payload & File Sizing Ceilings
- Enforced a hard 1 MiB (`1,048,576` bytes) ceiling on service specification files and inline JSON payloads in `aiosh service validate --spec <input>`.
- Any file or inline string exceeding 1 MiB is rejected with exit code 2 and structured error code `PAYLOAD_TOO_LARGE` prior to memory allocation or JSON parsing, emitting a failure audit row.
- In-memory data model enforces strict resource bounds:
  - `name`: max 128 bytes.
  - `exec_start`, `exec_stop`, `exec_reload`: max 4,096 bytes.
  - `description`: max 4,096 bytes.
  - `dependencies`: max 128 items.
  - `environment`: max 256 key-value pairs (keys and values capped at 4,096 bytes).
  - `timeout_start_secs`, `timeout_stop_secs`: strictly bounded within $[1 \dots 86,400]$ seconds.
  - `user`, `group`: max 32 bytes matching POSIX regex.

### 2. Explicit Result Envelopes
- Never fails silently. Every failure mode produces an explicit JSON error envelope containing:
  - `code`: non-zero integer (`1` for file not found / operational error, `2` for validation failure / argument error).
  - `data`: `null` on failure, structured result on success.
  - `error`: `{ "code": "VALIDATION_FAILED" | "PAYLOAD_TOO_LARGE" | "JSON_PARSE_ERROR" | "FILE_READ_ERROR" | "MISSING_ARGUMENTS" | "UNKNOWN_SUBCOMMAND", "message": "...", "errors": [...] }`.

### 3. Resource Cleanup & RAII Guarantees
- Zero dangling file handles or locks across error branches; files are read via standard scoped handles dropped immediately.
- SQLite connections are transiently acquired and released via RAII `Drop` during audit record persistence.

### 4. Honest Audit Trail
- All operations (valid names, valid specs, malformed specs, oversized payloads, file read failures, syntax errors) emit an honest audit row to the SQLite WAL ring (`audit.db`) with SHA-256 hash chaining via `classify_and_emit` per ADR-0035 §F-2.
