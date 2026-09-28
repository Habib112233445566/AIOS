# T-02432: Sandbox Enforcement MCP/API Surface Specification

## 1. Specification Overview
This specification defines the JSON-RPC 2.0 contract for exposing Sandbox Enforcement capabilities to autonomous agents via the Model Context Protocol (MCP) server (`code/aiosh-rust/aiosh-mcp`).

---

## 2. Tool Definitions & JSON Schemas

### A. Tool: `aios.sandbox.profiles`
- **Purpose**: List registered sandbox containment profiles.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {},
    "additionalProperties": false
  }
  ```
- **Response Format**:
  Array of serialized `SandboxProfile` objects with fields:
  `name`, `isolation_level`, `network`, `filesystem`, `syscalls`, `resources`, `env`.
- **Audit Logging**: Recorded as read-only inspection event in `audit_ring`.

### B. Tool: `aios.sandbox.probe`
- **Purpose**: Probe host kernel sandbox capabilities.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {},
    "additionalProperties": false
  }
  ```
- **Response Format**:
  Object containing:
  `landlock_supported` (bool), `landlock_abi_version` (int|null), `seccomp_bpf_supported` (bool), `no_new_privs_supported` (bool), `platform` (string).

### C. Tool: `aios.sandbox.exec`
- **Purpose**: Execute a command inside a supervised, bounded containment profile.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "command": { "type": "string", "description": "Absolute or PATH-relative executable path" },
      "args": { "type": "array", "items": { "type": "string" }, "description": "Command line arguments" },
      "profile": { "type": "string", "description": "Target profile name (standard, strict, permissive). Defaults to standard." },
      "cwd": { "type": "string", "description": "Working directory. Directory traversal (..) is prohibited." },
      "grant_token": { "type": "string", "description": "Optional PEP capability grant token for authorization" }
    },
    "required": ["command"],
    "additionalProperties": false
  }
  ```
- **Response Format**:
  `SandboxExecutionResult` object containing:
  `exit_code` (int), `status` (string), `stdout` (string), `stderr` (string), `duration_ms` (int), `components` (array).

---

## 3. Error Codes & Failure Modes
- `ERR_SANDBOX_EMPTY_COMMAND`: `command` parameter is blank or whitespace.
- `ERR_SANDBOX_INVALID_PATH`: Directory traversal (`..`) detected in `command` or `cwd`.
- `ERR_SANDBOX_PROFILE_NOT_FOUND`: Target profile name does not exist in registry.
- `ERR_SANDBOX_PEP_UNAUTHORIZED`: PEP authorization failed or missing grant token when enforcement active.
- `ERR_SANDBOX_EXEC_FAILED`: Target binary failed to spawn or execute.

---

## 4. Audit Invariants
All invocations of `aios.sandbox.exec` route strictly through `dispatch::recorded_call`, writing an immutable record to the SQLite WAL `audit_ring` table with tool name `"sandbox"`, binary target, duration, exit code, and PEP authorization grant.
