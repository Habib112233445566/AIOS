# T-01166 — Base Image Build / Security Policy: Integration

**Date:** 2026-09-04
**Type:** Integration
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Security Policy

## 1. Integration Scope
- **CLI Integration**:
  - Integrated `aiosh image policy [<id>] [--json] [--store <path>]` into `cmd_image` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
  - Audited via `classify_and_emit` into the SHA-256 hash-chained AuditRing.
  - Added unit/integration tests in `task_cli_tests::test_cmd_image_flow`.
- **MCP Tool Integration**:
  - Registered `aios.image.policy` tool in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
  - Dispatched via `dispatch::recorded_call` verifying PEP integration and structured JSON returns.
  - Added unit/integration tests in `tests::test_mcp_image_tools`.
- **Test Validation**:
  - `cargo test -p aiosh-cli -p aiosh-mcp` passed completely (20/20 CLI tests, 8/8 MCP tests).
