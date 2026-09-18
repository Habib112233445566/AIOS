# T-01446: User Session Bootstrap — Configuration: Integration

## Metadata
- **Task ID:** `T-01446`
- **Subsystem:** `code/aiosh-rust/aiosh-cli` and `code/aiosh-rust/aiosh-mcp`
- **Component:** User Session Bootstrap Configuration Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Integration Deliverables

1. **Operator CLI Surface (`aiosh session config`)**:
   - Integrated subcommand `aiosh session config [--config <path>] [--json]` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
   - Bounded argument validation: checks config path $\le 1024$ characters and control character rejection.
   - Standardized JSON envelope output (`{"code": 0, "data": ..., "error": null}`) and human-readable formatted output.
   - Audit logging: emits structured audit rows to SQLite WAL via `classify_and_emit` on every resolution.
   - Updated root CLI usage and `aiosh session --help` to advertise `config`.

2. **Autonomous Agent MCP Surface (`aios.session.config`)**:
   - Registered tool `aios.session.config` in `code/aiosh-rust/aiosh-mcp/src/main.rs` with JSON schema describing `config_path` and `grant_id`.
   - Wired dispatch via `dispatch::recorded_call` with PEP capability verification and monotonic hash-chained SQLite WAL audit logging.
   - Added automated unit test assertion in `aiosh-mcp` verifying tool discovery and execution.

## 2. Cross-Substrate Parity
- Verified both CLI and MCP surfaces resolve identical `SessionConfig` models from defaults, environment variables (`AIOS_SESSION_*`), and configuration files.
