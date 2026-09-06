# T-01338: Init & Service Supervision - MCP/API Surface: Hardening

## Metadata
- **Task ID:** `T-01338`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Init & Service Supervision MCP / API Surface Hardening
- **Status:** Complete (Hardening)

---

## 1. Hardening Overview & Defenses
This task verifies and documents the hardening mechanisms protecting the MCP service supervision API surface against Denial of Service (DoS), memory exhaustion, resource leaks, and unlogged failures.

---

## 2. Hardening Measures Implemented

### 2.1 Explicit Size Caps & Input Boundaries
- **Service Name Length**: Strictly capped at 128 characters (`name.len() <= 128`).
- **Query Filter Pattern**: Capped at 256 characters (`pattern.len() <= 256`).
- **Store Path Length**: Capped at 1024 characters (`store_path.len() <= 1024`).
- **Executable Paths**: Capped at 4096 characters in `ServiceSpec` validation.
- **Dependency List**: Capped at 128 dependencies per service unit.
- **Control Character Rejection**: All string inputs validate `!c.is_control()`, rejecting any injection attempts.

### 2.2 Standard Error Envelope (No Silent Failures)
- Every error condition across parameter extraction, syntax checks, FSM execution, or store persistence returns a structured JSON envelope:
  ```json
  {
    "ok": false,
    "error": "<diagnostic message>",
    "tool": "<tool_name>"
  }
  ```
- No silent fallback or unlogged exception paths. Handlers return `Result<Value, String>` without panics.

### 2.3 Resource Cleanup & Tempfile Isolation
- Store persistence in `save_to_path` isolates temporary files using process IDs:
  `parent.join(format!(".{}.tmp.{}", file_name, std::process::id()))`
- Atomic rename (`fs::rename`) ensures writes are all-or-nothing.
- On any I/O failure during file creation or serialization, the temporary file is immediately cleaned up (`let _ = fs::remove_file(&tmp_path)`).
- File descriptors are dropped immediately at end-of-scope.

### 2.4 ADR-0035 §F-2 Honest Audit Logging
- Every execution of `dispatch::recorded_call` invokes `classify_and_emit` to record the event into the tamper-evident `AuditRing`.
- Both successful actions and rejected/failed calls emit audit rows with full actor attribution and tool arguments.

---

## 3. Hardening Verification
- Verified by unit tests in `test_mcp_service_tools` (`aiosh-mcp`).
- Verified by end-to-end Python smoke suite `test_service_mcp_smoke.py`.
- Verified subsystem suite `test_service_suites.py`.
- Zero temp file leaks or open connection leaks observed under failure conditions.
