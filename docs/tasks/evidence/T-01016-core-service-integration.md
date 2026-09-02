# T-01016 — Distro Selection & Justification / Core Service: Integration

## 1. Production Surfaces Integrated
- **CLI Subcommand Surface (`aiosh distro`)**:
  - `aiosh distro list [--json] [--store <path>]`: Lists all registered distro profiles (`debian-12-minimal-x86_64`, `alpine-319-container-x86_64`) with architecture and recommended status.
  - `aiosh distro show <id> [--json] [--store <path>]`: Displays detailed profile metadata including kernel requirements, default package payloads, init system, and justification.
  - `aiosh distro evaluate [<id>] [--json] [--store <path>]`: Executes criteria scoring engine across binary compatibility, footprint, and security metrics.
  - `aiosh distro recommend [--json] [--store <path>]`: Returns the designated production reference profile.
- **MCP Server Tool Surface (`aiosh-mcp`)**:
  - `aios.distro.list`: Dispatches through `dispatch::recorded_call` and audit logging.
  - `aios.distro.show`: Inspects specific distro profiles.
  - `aios.distro.evaluate`: Evaluates distro profiles against AIOS criteria.
  - `aios.distro.recommend`: Recommends base distribution profile.

## 2. Automated Integration Test Coverage
- `test_cmd_distro_flow`: End-to-end Rust CLI smoke execution in `aiosh-cli`.
- `test_mcp_distro_tools`: Full MCP tool execution in `aiosh-mcp`.
- `tools/test_distro_suites.py`: Extended with criteria `D3` (CLI surface) and `D4` (MCP surface).

## 3. Verification Output
```
[+] D1 distro data model integrity & validation invariants
[+] D2 distro store lifecycle, registry querying & persistence
[+] D3 distro CLI surface commands & options (list/show/evaluate/recommend)
[+] D4 distro MCP tools dispatch & execution (list/show/evaluate/recommend)

PASS: distro_suites criteria (D1..D4)
```
