# T-01196: Base Image Build Recovery & Validation Integration

**Date:** 2026-09-04  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Base Image Build / Recovery & Validation  
**Task ID:** T-01196  

## 1. Integration Scope & Changes
Integrated the Recovery & Validation subsystem across production CLI and MCP tool call paths:
1. **CLI Integration (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
   - Added subcommand `aiosh image check [--fix] [--json] [--store <path>]`.
   - In standard mode, validates on-disk registry and exits 0 on healthy or 1 on errors.
   - In fix mode (`--fix`), invokes `load_or_recover`, performs non-destructive backup, restores clean defaults, and exits 0.
   - Emits structured audit event to SQLite WAL via `classify_and_emit`.
   - Verified via `test_cmd_image_flow` in `aiosh-cli`.
2. **MCP Tool Integration (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
   - Registered tool `aios.image.check` with optional `store_path` and `auto_recover` parameters.
   - Wired tool handler through `dispatch::recorded_call` for SHA-256 audit-chained logging.
   - Verified via `test_mcp_image_tools` in `aiosh-mcp`.
3. **Smoke Verification**:
   - Both CLI (20/20) and MCP (8/8) test suites pass with zero regressions.
   - Existing image suite criteria (`B1..B8`) pass.
