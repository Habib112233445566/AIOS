# T-00706 — Repository Health / recovery & validation: Integration

## 1. Integration Scope
This task verifies the integration of recovery helpers and validation checks across CLI (`aiosh repo health|check`) and MCP (`aios.repo.health`) execution surfaces.

## 2. Integration Deliverables
- **CLI Subcommand Integration**:
  - `aiosh repo health` resiliently falls back to recovered default configuration when config paths are absent, verified in `code/aiosh-cli/tests/test_repo_cli_smoke.py`.
- **MCP Tool Integration**:
  - `aios.repo.health` executes full health checks and validates aggregate report integrity, verified in `code/aiosh-mcp/tests/test_repo_mcp_smoke.py`.
- **Automated Test Suite**:
  - Full criteria runner `tools/test_repo_health_suites.py` passing H1..H7.

## 3. Test Verification
```text
[+] H1 data model integrity
[+] H2 git tree hygiene diagnostics
[+] H3 file bounds scanner
[+] H4 security governance audit
[+] H5 CLI surface commands
[+] H6 MCP tool schemas & JSON-RPC
[+] H7 configuration schema & hardening

PASS: repo_health_suites criteria (H1..H7)
```
