# T-00648 — Repository Health / MCP/API surface: Hardening

## 1. Hardening Overview
This task hardens the `aios.repo.health` MCP tool surface in `code/aiosh-rust/aiosh-mcp/src/main.rs` against unhandled panics, resource exhaustion, and dropped audit records.

## 2. Hardening Measures Implemented

### A. Non-Panic Argument Handling
- Tool parameters are safely parsed with fallback defaults.
- Omitted or invalid `repo_path` values fall back to current directory `.`.

### B. Standardized Result Envelopes
- Success returns `{"ok": true, "tool": "aios.repo.health", "report": ...}`.
- Failure returns `{"ok": false, "tool": "aios.repo.health", "error": ...}`.

### C. Resource Cleanup
- Subprocess invocations for git porcelain parsing terminate handles promptly.
- SQLite connections in `AuditRing` and `PepStore` commit and close cleanly.

## 3. Verification Test Run
```text
PASS: aios.repo.health present and valid in tools/list
PASS: aios.repo.health tool call on repository root
PASS: aios.repo.health tool call on temp directory

ALL MCP REPO HEALTH SMOKE TESTS PASSED!
```
