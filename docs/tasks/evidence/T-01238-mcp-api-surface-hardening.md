# T-01238: Package Management - MCP/API Surface: Hardening

## Metadata
- **Task ID:** `T-01238`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Package Management MCP/API Surface Hardening
- **Status:** Complete

## 1. Hardening Overview
The Package Management MCP/API surface has been hardened against failure modes, malformed payloads, excessive resource allocation, and edge cases across all six tools (`aios.package.validate`, `aios.package.list`, `aios.package.get`, `aios.package.plan`, `aios.package.search`, `aios.package.apply`).

## 2. Hardening Measures Implemented

### A. Strict Input Length, Boundary, and Control-Character Checking
1. **Package Name Constraints (`validate`, `get`)**:
   - Length capped at 128 characters max.
   - Rejection of any control characters (`c.is_control()`).
   - PM1 POSIX naming syntax verification (`^[a-z0-9][a-z0-9+.-]{1,63}$`).
2. **Search and Filter Pattern Constraints (`list`, `search`)**:
   - Pattern length strictly bounded to $\le 256$ characters.
   - Rejection of control characters, null bytes (`\0`), and line breaks.
   - Exact substring comparison (`to_lowercase().contains(...)`) avoiding regular expression engine overhead and ReDoS vulnerabilities.
3. **Limit Pagination Bounds (`list`, `search`)**:
   - Enforced integer range `1..=10,000`.
   - Explicit error returned when `limit == 0` or `limit > 10,000`.
4. **Action List Constraints (`plan`, `apply`)**:
   - Array length capped at 10,000 elements to prevent unbounded memory allocation and CPU cycles during dependency closure calculations.
5. **Store Path Bounds (`list`, `get`, `plan`, `search`, `apply`)**:
   - Path strings checked for maximum length of 1,024 characters.
   - Immediate rejection if control characters or null bytes are present.

### B. Standard Result Envelope & Non-Silent Failure (ADR-0035)
- All tool endpoints return a structured JSON object `{ "ok": bool, ... }`.
- Under no circumstances does an execution fail silently or panic.
- Any validation or runtime failure inside the tool closure yields `Err(msg)`, which `dispatch::recorded_call` maps to:
  ```json
  {
    "ok": false,
    "tool": "<tool_name>",
    "audit_id": <id>,
    "reason": "<error_message>",
    "gate": "<classifier|pep|null>",
    "policy_revision": "<rev>"
  }
  ```
- Every refusal or execution failure writes an honest audit row to the `AuditRing` with outcome `refused` or `failure`, including full error diagnostics.

### C. Resource Cleanup & Leak Prevention
- **Temporary Files**: `PackageStore::save_to_path` writes to an adjacent `.tmp` file and replaces the destination file via atomic rename. On any I/O failure or serialization error, `std::fs::remove_file(&tmp_path)` is executed to prevent lingering temporary files.
- **Store Size Ceilings**: `PackageStore::load_from_path` verifies file size metadata before reading, enforcing a 10 MiB hard cap via `take(10 * 1024 * 1024 + 1)` on stream reading.
- **Process & Socket Isolation**: Tools run entirely in-process without spawning subshells or holding background network sockets.

## 3. Test Coverage & Verification
Expanded `test_mcp_package_tools` to 28 comprehensive assertions covering:
- Control character and zero/oversized limit rejection in `aios.package.list`.
- Control character and oversized name rejection in `aios.package.get`.
- Invalid store path rejection with control characters across `list`, `get`, `plan`, `search`, and `apply`.
- Atomic persistence and disk round-trips.

All tests pass cleanly in the cargo test runner and `tools/test_package_suites.py`.
