# T-01828: Network Bootstrap / CLI Surface: Hardening

## 1. Overview
- **Task ID**: `T-01828`
- **Sub-Epic**: 3 (CLI Surface)
- **Goal**: Harden the CLI surface of Network Bootstrap against failure, resource exhaustion, and misuse.

---

## 2. Hardening Measures Implemented

### 1. Strict Size Bounds & Input Constraints
- **Path Length Caps**: Flag arguments (`--sysfs`, `--procfs`, `--resolv`) are constrained to $\le 1024$ characters.
- **Control Character Scrubbing**: Rejects any path string containing ASCII/Unicode control characters (`c.is_control()`).
- **Interface Name Caps**: Constrained to $\le 15$ characters with strict character allowlisting (`^[a-zA-Z0-9_.-]+$`).
- **Bounded Underlying Service Reads**: Integrates with `NetworkService::read_bounded_string` capping sysfs reads to 64 KB, route tables to 1 MB, and resolv.conf to 64 KB.

### 2. Standardized Error Reporting (Zero Silent Failures)
- Every error condition returns a non-zero exit code:
  - Exit code `1`: Operational error (e.g. `INTERFACE_NOT_FOUND`, `LIST_FAILED`, `ROUTES_FAILED`, `DNS_FAILED`, `STATE_FAILED`, `OPERATION_FAILED`).
  - Exit code `2`: Syntax, argument, or validation error (e.g. `UNKNOWN_SUBCOMMAND`, `PATH_TOO_LONG`, `PATH_CONTAINS_CONTROL_CHAR`, `MISSING_INTERFACE_NAME`, `INVALID_INTERFACE_NAME`).
- With `--json`, error responses are wrapped in a deterministic envelope:
  ```json
  {
    "code": 2,
    "data": null,
    "error": {
      "code": "INVALID_INTERFACE_NAME",
      "message": "invalid interface name 'eth0;bad': interface name must match ^[a-zA-Z0-9_.-]+$"
    }
  }
  ```

### 3. Resource Hygiene & Leak Prevention
- CLI execution is stateless with RAII cleanup: `AppContext` automatically commits/closes database connections on scope exit.
- No temporary files or background processes are spawned during command execution.

### 4. Honest Audit Logging
- Every single branch (including error branches such as invalid flags or unknown subcommands) invokes `classify_and_emit` to guarantee complete audit coverage per ADR-0035 §F-2.
