# T-01346: Init & Service Supervision - Configuration: Integration

## Metadata
- **Task ID:** `T-01346`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`, `code/aiosh-rust/aiosh-mcp`
- **Component:** Init & Service Supervision Configuration Integration
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Integration Overview

This task wires the Init & Service Supervision Configuration Subsystem (`aiosh-core::service_config`) into both operator CLI and Autonomous Agent MCP production surfaces:

1. **CLI Surface (`aiosh service config`)**:
   - In `code/aiosh-rust/aiosh-cli/src/main.rs`:
     - Added `config` subcommand to `cmd_service`.
     - Supports `--config <path>` flag and `--json` flag.
     - Validates configuration path bounds ($<= 1024$ chars, no control characters).
     - Resolves configuration via `ServiceConfig::resolve`.
     - Records audit trail via `classify_and_emit` to `AuditRing`.
     - Formats human-readable table or structured JSON result envelope.
     - Added `config` entry to `aiosh service --help`.

2. **Autonomous Agent MCP Surface (`aios.service.config`)**:
   - In `code/aiosh-rust/aiosh-mcp/src/main.rs`:
     - Registered `aios.service.config` in tool manifest with input schema (`config_path`, `grant_id`).
     - Added dispatch routing via `dispatch::recorded_call` ensuring PEP verification and audit entry.
     - Returns `{ "ok": true, "tool": "aios.service.config", "config": ... }`.
     - Auto-discovered by `ai_agent.py` dynamic tool routing on "service" keyword.

---

## 2. Verification & Acceptance Criteria
- [x] CLI production surface `aiosh service config [--json] [--config <path>]` reachable and working.
- [x] MCP tool `aios.service.config` registered in tool catalog and returning valid structured configuration.
- [x] Audit trails recorded in SQLite WAL ring for configuration queries.
- [x] End-to-end test suite (`cargo test -p aiosh-mcp test_service_tools`) passes.
