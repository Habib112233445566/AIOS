# Task Evidence: T-02638 — Secrets Handling MCP/API Surface Hardening

## 1. Summary
Hardened the Model Context Protocol (MCP) tool surface for Secrets Handling in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
1. **Size Ceilings & Bounds Enforcement**:
   - `MAX_SECRET_PAYLOAD_SIZE` (64 KiB) enforced on `aios.secret.store` and `aios.secret.rotate`.
   - Store file size ceiling (1 MiB) verified via `symlink_metadata` in `load_safe_secret_service`.
   - Identifier length limit (128 characters) and control character rejection on `id`.
2. **Explicit Error Envelopes**:
   - Standardized error codes across all 5 MCP tools:
     - `ERR_SECRET_INVALID_INPUT`
     - `ERR_SECRET_PAYLOAD_TOO_LARGE`
     - `ERR_SECRET_PATH_TRAVERSAL`
     - `ERR_SECRET_INVALID_KIND`
     - `ERR_SECRET_INVALID_SCOPE`
     - `ERR_SECRET_NOT_FOUND`
     - `ERR_SECRET_ACCESS_DENIED`
     - `ERR_SECRET_REVOKED`
   - Every failure path emits structured `{ "ok": false, "error": ... }` JSON objects. Silent failures are strictly eliminated.
3. **Leak-Free Resource Cleanup**:
   - Vault saving relies on `SecretService::save_to_path`, which implements atomic temporary file rename (`.tmp.{pid}.{nanos}`) with guaranteed error-path cleanup.
4. **Audit Consistency**:
   - All tool calls, both successes and rejections, are logged through `dispatch::recorded_call` into `AuditRing`.

## 2. Test Verification
- All tests in `code/aiosh-rust/aiosh-mcp` passed (`cargo test -p aiosh-mcp test_mcp_secret_tools_execution`).
- All tests in `code/aiosh-mcp/tests/test_secret_mcp.py` passed.
- Workspace compiles with 0 errors and 0 warnings.
