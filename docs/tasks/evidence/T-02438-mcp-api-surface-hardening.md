# T-02438: Sandbox Enforcement MCP/API Surface Hardening

## 1. Hardening Scope
This document details the hardening protections implemented for the Sandbox Enforcement MCP/API surface (`aiosh-mcp`).

## 2. Hardening Measures

### A. Size Capping & Bounded Memory Buffers
- Process standard output and standard error streams are strictly clamped to `DEFAULT_MAX_OUTPUT_CAPTURE_BYTES = 10 MiB`.
- Execution arguments are capped at `MAX_ARGS_COUNT = 1024`.
- Command executable paths are restricted to `MAX_PATH_LEN = 4096`.

### B. Explicit, Fail-Closed Error Envelopes
Errors encountered during MCP parameter parsing, validation, or execution return explicit JSON-RPC error frames containing structured codes:
- `ERR_SANDBOX_EMPTY_COMMAND`
- `ERR_SANDBOX_INVALID_PATH`
- `ERR_SANDBOX_PROFILE_NOT_FOUND`
- `ERR_SANDBOX_PEP_UNAUTHORIZED`

### C. Resource Cleanup Guarantee
- Subprocess handles are synchronously awaited to completion and dropped immediately to prevent process leaks or zombies.
- SQLite WAL transactions are scoped strictly within `dispatch::recorded_call`, preventing dangling database locks.

### D. Audit Logging on Error
Per ADR-0035 §F-2, all failures, rejections, and invalid calls record an honest audit row with status `"failure"` and error details before returning the error response to the client.
