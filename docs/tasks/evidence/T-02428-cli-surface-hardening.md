# T-02428: Sandbox Enforcement CLI Surface Hardening

## 1. Hardening Overview
This document records the hardening implementations applied to the Sandbox Enforcement CLI surface (`aiosh sandbox` / `aiosh sb` in `code/aiosh-rust/aiosh-cli/src/main.rs`).

## 2. Hardening Measures Implemented

### A. Non-Silent Fail-Closed Error Reporting
Every error path in `cmd_sandbox` produces an unambiguous response:
- In human mode: Formatted and sanitized error message to `stderr` with non-zero exit code (`1`, `2`, or `127`).
- In JSON mode: Uniform envelope matching AIOS CLI specification:
  ```json
  {
    "code": 2,
    "data": null,
    "error": {
      "code": "TRAVERSAL_DETECTED",
      "message": "directory traversal prohibited in cwd: ../outside"
    }
  }
  ```

### B. Universal Audit Row Emission (ADR-0035 §F-2)
Every execution path (success, rejection, invalid arguments, child failure) writes exactly one structured row to `audit_ring`:
- Outcome `"success"` when exit code is `0`.
- Outcome `"failure"` when exit code is non-zero, recording the specific failure reason and parameters.

### C. Resource Cleanup & Child Process Supervision
- Child processes are supervised synchronously to termination in `SandboxService::execute`; process handles are dropped immediately to prevent handle leaks.
- Database connections to SQLite WAL are scoped to execution blocks with automatic connection closure upon function return.
- Captured memory buffers are hard-capped at 10 MiB, preventing unbounded memory accumulation.

### D. Safe Control Character Sanitization
- `sanitize_terminal_output` strips escape sequences while preserving standard newline and tabulation semantics, eliminating terminal hijacking vulnerabilities without breaking standard output formatting.
