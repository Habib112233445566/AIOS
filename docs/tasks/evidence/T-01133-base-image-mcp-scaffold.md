# T-01133 — Base Image Build / MCP/API Surface: Scaffold

**Date:** 2026-09-03
**Type:** Scaffold
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / MCP/API Surface

## 1. Scaffold Deliverables
- Registered tools `aios.image.list`, `aios.image.get`, `aios.image.plan` in `Server::tool_manifest` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
- Added handler skeletons in `Server::call_tool` routing through PEP and audit recording.
- Verified workspace compilation (`cargo check`).
