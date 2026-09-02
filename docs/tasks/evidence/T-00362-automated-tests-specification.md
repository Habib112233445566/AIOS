# T-00362 — Dependency & Toolchain Pinning / automated tests: Specification

## Goal
Formalize the automated test boundaries for the Dependency & Toolchain Pinning epic.

## Specification

### 1. CLI Smoke Test (`code/aiosh-cli/tests/test_toolchain_cli_smoke.py`)
- **Purpose**: Verify `aiosh toolchain check` and `aiosh toolchain config`.
- **Inputs**: Spawns `aiosh.exe` via `subprocess.Popen`.
- **Assertions**:
  - `aiosh toolchain config` exits 0 and returns the valid JSON payload matching `config/toolchain.json`.
  - `aiosh toolchain check` exits 0 and returns `ok: true`.

### 2. MCP Smoke Test (`code/aiosh-mcp/tests/test_toolchain_mcp_smoke.py`)
- **Purpose**: Verify `aios.toolchain.check` and `aios.toolchain.config.get` via JSON-RPC.
- **Inputs**: Spawns `aiosh-mcp.exe` via `subprocess.Popen` and writes JSON to `stdin`.
- **Assertions**:
  - `aios.toolchain.config.get` returns `ok: true` and the config map.
  - `aios.toolchain.check` returns `ok: true`.
  - Check the output for syntax valid JSON-RPC envelopes and `isError: false`.

### 3. CI Orchestrator (`tools/ci_suites.py`)
- Append `toolchain_cli_smoke` and `toolchain_mcp_smoke` to the `SUITES` list, maintaining the registry contract for deterministic ordering.
