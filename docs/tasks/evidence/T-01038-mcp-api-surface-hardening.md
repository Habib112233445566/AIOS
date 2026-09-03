# T-01038 — Distro Selection & Justification / MCP/API Surface: Hardening

## 1. Hardening Deliverables
- **Structured Error Envelope**:
  - Rejection paths emit `{ "ok": false, "error": "<reason>" }` with `isError: true`, preventing unhandled JSON-RPC aborts.
  - Negative responses tested and confirmed for missing parameters and non-existent IDs.
- **Audit Logging of Failures**:
  - Failed invocations unconditionally emit an audit event into `AuditRing` with `"outcome": "error"` or `"failure"`.
- **Resource Bounds**:
  - File reading via `store_path` is capped at `MAX_STORE_BYTES` (10 MiB).
  - No connection or temp file leaks on error paths.

## 2. Test Verification
Verified using `code/aiosh-mcp/tests/test_distro_mcp_smoke.py`:
```
PASS: aiosh-mcp tools/call aios.distro.show missing id rejected with error envelope
PASS: aiosh-mcp tools/call aios.distro.show nonexistent profile returns ok: false
ALL DISTRO MCP SMOKE TESTS PASSED!
```
