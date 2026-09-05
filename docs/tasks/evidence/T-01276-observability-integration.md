# T-01276: Package Management / Observability - Integration

## Overview
Task `T-01276` integrates the Package Management Observability Subsystem across all operational planes in AIOS:
1. **CLI Surface**: Added `aiosh package stats [--store <path>] [--config <path>] [--json]` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
2. **MCP API Surface**: Registered `aios.package.stats` tool in `tool_manifest`, added invocation dispatch handler with SQLite WAL audit logging via `dispatch::recorded_call`, and expanded `test_mcp_package_tools` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
3. **Master Package Runner**: Added criterion `PM8` (`package observability telemetry report & invariants (PO1..PO6)`) to `tools/test_package_suites.py`.

## CLI Integration
- Subcommand: `aiosh package stats`
  - `--store <path>`: Specifies custom package store file path.
  - `--config <path>`: Specifies custom policy JSON file path for compliance evaluation.
  - `--json`: Outputs structured JSON telemetry report.
  - Formatted human output displays total packages, installed storage footprint, average size, policy compliance rates, and breakdowns across states, formats, architectures, and dependencies.
- Verified in `aiosh-cli::task_cli_tests::test_cmd_package_flow`:
  - `aiosh package stats` -> Exit code 0
  - `aiosh package stats --json` -> Exit code 0
  - `aiosh package stats --store bad\0store` -> Exit code 2 (rejected control chars)

## MCP Tool Integration
- Tool: `aios.package.stats`
  - Registered in `Server::tool_manifest()`.
  - Properties: `store_path`, `config_path`, `grant_id`.
  - Audited via `dispatch::recorded_call` into PEP ring buffer.
  - Verified in `aiosh-mcp::tests::test_mcp_package_tools`:
    - Calling `aios.package.stats` returns `ok: true` and `report.total_packages > 0`.
    - Passing control characters in `store_path` returns `ok: false`.
    - Manifest discovery confirms `aios.package.stats` tool advertisement.

## Master Runner Integration (PM8)
Updated `tools/test_package_suites.py`:
```python
def test_pm8_observability():
    return _run_cargo_test(
        ["--test", "test_package_observability"],
        "PM8",
        "package observability telemetry report & invariants (PO1..PO6)",
    )
```

Execution Output:
```
[+] PM1 package data model integrity & invariants (PM1..PM5)
[+] PM2 package core service integrity & invariants (CS1..CS5)
[+] PM3 package CLI surface commands & options (validate/list/show/search/plan/apply)
[+] PM4 package MCP tool surface (validate/list/get/plan/search/apply)
[+] PM5 package configuration resolution & invariants (PC1..PC6)
[+] PM6 package automated integration test matrix (PT1..PT6)
[+] PM7 package security policy evaluation & invariants (PP1..PP6)
[+] PM8 package observability telemetry report & invariants (PO1..PO6)

PASS: package_suites criteria (PM1..PM8)
```
