# T-00696 — Repository Health / documentation: Integration

## 1. Integration Scope
This task verifies the integration of `format_repo_health_summary` and health reporting across CLI (`aiosh repo health|check`) and MCP (`aios.repo.health`) surfaces.

## 2. Integration Deliverables
- **CLI Subcommand Surface**:
  - `aiosh repo health` and `aiosh repo check` producing human-readable and structured `--json` diagnostics.
  - Smoke tests verified in `code/aiosh-cli/tests/test_repo_cli_smoke.py`.
- **MCP Server Surface**:
  - `aios.repo.health` tool registration and JSON-RPC dispatch verified in `code/aiosh-mcp/tests/test_repo_mcp_smoke.py`.

## 3. Test Verification
```text
PASS: aiosh repo health prose output
PASS: aiosh repo health --json output
PASS: aiosh repo check alias
PASS: aiosh repo health --repo custom path
PASS: aiosh repo invalid subcommand rejection

ALL REPO CLI SMOKE TESTS PASSED!
PASS: aios.repo.health present and valid in tools/list
PASS: aios.repo.health tool call on repository root
PASS: aios.repo.health tool call on temp directory

ALL MCP REPO HEALTH SMOKE TESTS PASSED!
```
