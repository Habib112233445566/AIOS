# Task Evidence: T-02138 (MCP/API Surface: Hardening)

## Overview
- **Task ID**: `T-02138`
- **Task Name**: MCP/API surface: Hardening
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 4: MCP / API Surface
- **Timestamp**: 2026-09-21T00:53:50+05:00
- **Status**: COMPLETED

## Hardening Controls Applied

1. **Input Size & Boundary Caps**:
   - `store_path`: Clamped to $\le 1024$ characters; control characters and null bytes strictly rejected.
   - `id`: Bounded to $1 \le \text{len} \le 128$ characters; control characters strictly rejected.
   - `subject` / `resource` / `action`: Bounded strings with sanitization in core engine.
   - `effect`: Whitelisted strictly to `"permit"` or `"deny"` (case-insensitive conversion).
   - Registry capacity: Hard ceiling of 5,000 rules (`MAX_RULES_IN_SERVICE`).
   - Disk file size: Hard ceiling of 10 MB (`MAX_PEP_SERVICE_STORE_SIZE`).

2. **Standard Result Envelopes & Zero Silent Failures**:
   - Every failure mode (validation error, not found, capacity reached, IO error) returns an explicit standard error response via JSON-RPC.
   - No silent drops or swallowing of errors.
   - Invocations returning error still record an audit event in the SQLite audit ring with `status: "error"` and the error message, ensuring complete observability.

3. **Atomic File Persistence & Resource Hygiene**:
   - Policy file writes utilize atomic rename (`.tmp` write followed by `fs::rename`) to prevent partial or corrupted file writes.
   - Symlinks are explicitly rejected before read/write operations.
   - Temporary files are cleanly cleaned up on write errors.
   - Corrupted files undergo non-destructive quarantine (`.bak.<timestamp>` with restricted `0600` permissions).

4. **Fail-Closed Authorization**:
   - In any ambiguous evaluation state or missing rule scenario, authorization decisions resolve to `deny` (`allowed: false`).

## Verification
- Verified against `code/aiosh-mcp/tests/test_pep_decision_smoke.py`:
  - Path traversal rejection tested (`../../../etc/shadow.json`).
  - Non-existent rule deletion error handling tested (`nonexistent_rule`).
  - Clean error propagation verified across all endpoints.
