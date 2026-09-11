# T-01449: User Session Bootstrap — Configuration: Documentation

## Metadata
- **Task ID:** `T-01449`
- **Subsystem:** `code/aiosh-cli` and `code/aiosh-mcp`
- **Component:** User Session Bootstrap Configuration Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Documentation Deliverables

1. **CLI Documentation (`code/aiosh-cli/README.md`)**:
   - Documented `aiosh session config` subcommand, option flags (`--config <path>`, `--json`), and output format.
   - Provided copy-pasteable CLI commands.
   - Documented environment variables (`AIOS_SESSION_STORE_PATH`, `AIOS_SESSION_MAX_PER_USER`, `AIOS_SESSION_MAX_TOTAL`, `AIOS_SESSION_IDLE_TIMEOUT_SECS`, `AIOS_SESSION_MAX_STORE_SIZE_BYTES`, `AIOS_SESSION_AUTO_PERSIST`).

2. **MCP Documentation (`code/aiosh-mcp/README.md`)**:
   - Registered tool `aios.session.config` in exposed tools table.
   - Added JSON-RPC 2.0 request/response schema specifications.

3. **Constraints & Known Limitations**:
   - Configuration files are limited to $\le 64\text{ KiB}$ ($65,536$ bytes). Files exceeding this size are rejected with `SC7 violation`.
   - Store paths must not contain ASCII control characters or null bytes (`\0`) and are capped at $1024$ bytes (`SC1`).
   - Session capacity limits are strictly validated against $[1 \dots 128]$ per user (`SC2`) and $[10 \dots 10,000]$ total (`SC3`).
   - Idle timeouts are bounded within $[10 \dots 86,400]$ seconds (`SC4`).
