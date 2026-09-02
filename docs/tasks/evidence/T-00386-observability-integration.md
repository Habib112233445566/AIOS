# T-00386 — Dependency & Toolchain Pinning / observability: Integration

## 1. Integration Scope
This task integrates the toolchain telemetry and diagnostics collectors into the primary CLI (`aiosh toolchain check`/`show`) and MCP server (`aios.toolchain.check`/`config.get`) entrypoints.

## 2. Integration Details
- **CLI & MCP Telemetry Wiring**:
  - `aiosh toolchain check` and `aios.toolchain.check` execute the underlying verification logic and emit detailed `outcome_detail` logs capturing parsed host versions and mismatch reasons directly to stdout/JSON-RPC and SQLite WAL audit records.
  - `aiosh toolchain show` and `aios.toolchain.config.get` serialize provenance-annotated manifest metadata (`source: "default" | "file" | "env"`), enabling operators and monitoring subagents to audit configuration origins.
- **Cross-Substrate Parity**:
  - Both CLI and MCP surfaces emit standardized JSON envelopes matching the canonical specification.

## 3. Verification
- `python code/aiosh-cli/tests/test_toolchain_cli_smoke.py` -> PASS (7/7 tests)
- `python code/aiosh-mcp/tests/test_toolchain_mcp_smoke.py` -> PASS (3/3 tests)
