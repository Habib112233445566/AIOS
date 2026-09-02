# T-00365 — Dependency & Toolchain Pinning / automated tests: Unit Test

## 1. Overview
This task enhances and exercises the standalone automated test suites for the Dependency & Toolchain Pinning epic covering both the CLI surface (`aiosh toolchain`) and the MCP JSON-RPC surface (`aios.toolchain.*`).

## 2. Test Coverage & Cases

### CLI Suite (`code/aiosh-cli/tests/test_toolchain_cli_smoke.py`)
- **Valid Cases**:
  - `test_toolchain_show`: Invokes `aiosh toolchain show`, validates `ok: true`, `subcommand: "toolchain show"`, and verified presence of `rust_version`, `python_version`, `node_version`, and `enforce_hashes`.
  - `test_toolchain_check`: Invokes `aiosh toolchain check` against the active host environment, asserting successful validation.
  - `test_toolchain_custom_config_valid`: Creates a temporary valid manifest and asserts `--config <path>` resolves and executes properly.
- **Negative & Boundary Cases**:
  - `test_toolchain_invalid_subcommand`: Asserts non-zero exit code on unrecognized subcommands.
  - `test_toolchain_missing_config`: Asserts non-zero exit code and structured error JSON envelope on non-existent config path.
  - `test_toolchain_corrupted_config`: Asserts failure on malformed JSON configuration.
  - `test_toolchain_mismatch_fails`: Asserts failure when pinned toolchain requirement does not match host binary version (e.g. `rust_version: "999.99.99"`).

### MCP Suite (`code/aiosh-mcp/tests/test_toolchain_mcp_smoke.py`)
- **Valid Cases**:
  - `test_mcp_config`: Asserts `aios.toolchain.config.get` returns valid JSON-RPC tool response with toolchain configuration schema.
  - `test_mcp_check`: Asserts `aios.toolchain.check` enforces host environment requirements and returns `ok: true`.
- **Negative Cases**:
  - `test_mcp_unknown_tool_fails`: Asserts error envelope on non-existent tool invocation (`aios.toolchain.nonexistent`).

## 3. Verification Output

### CLI Test Run:
```
PASS: aiosh toolchain show
PASS: aiosh toolchain check
PASS: aiosh toolchain custom config valid
PASS: aiosh toolchain invalid subcommand
PASS: aiosh toolchain missing config negative test
PASS: aiosh toolchain corrupted config negative test
PASS: aiosh toolchain version mismatch negative test
PASS: test_toolchain_cli_smoke.py
```

### MCP Test Run:
```
PASS: aios.toolchain.config.get
PASS: aios.toolchain.check
PASS: aios.toolchain unknown tool negative test
PASS: test_toolchain_mcp_smoke.py
```
