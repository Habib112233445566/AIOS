# T-01266: Package Management / Security Policy - Integration

## Overview
Task `T-01266` integrates the package security policy subsystem (`PackageSecurityPolicy`, `PackagePolicyMode`, `PackagePolicyVerdict`, and invariants `PP1..PP6`) across the complete AIOS operational plane:
1. **CLI Surface**: Integrated `aiosh package policy [--config <path>] [--package <name>] [--json]` subcommand into `code/aiosh-rust/aiosh-cli/src/main.rs`.
2. **MCP API Surface**: Registered `aios.package.policy` tool schema in `tool_manifest`, implemented structured execution handler with PEP audit logging, and added comprehensive verification assertions in `test_mcp_package_tools` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
3. **Master Package Test Suite**: Integrated criterion `PM7` (`package security policy evaluation & invariants (PP1..PP6)`) into `tools/test_package_suites.py`.

## CLI Integration
- Subcommand: `aiosh package policy`
  - `--config <path>`: Specifies custom policy configuration file path (validated <= 1024 chars, no control chars).
  - `--package <name>`: Evaluates a target package specification or prohibited list membership against the active policy.
  - `--json`: Formats policy report or policy evaluation verdict as machine-readable JSON.
  - Exit code 0 on compliance/success; exit code 2 on policy denial or validation error.
- Verified in `code/aiosh-rust/aiosh-cli/src/main.rs` (`test_cmd_package_flow`):
  - `aiosh package policy` -> Exit 0
  - `aiosh package policy --json` -> Exit 0
  - `aiosh package policy --package curl` -> Exit 0 (allowed)
  - `aiosh package policy --package telnet` -> Exit 2 (denied - PP2 violation)
  - `aiosh package policy --package nonexistent` -> Exit 2 (package not found)

## MCP Tool Integration
- Tool: `aios.package.policy`
  - Registered in `Server::tool_manifest()`.
  - Arguments: `config_path` (optional string), `package_name` / `package` (optional string), `store_path` (optional string).
  - Emits audit records to security ring buffer via `dispatch::recorded_call`.
  - Verified in `code/aiosh-rust/aiosh-mcp/src/main.rs` (`test_mcp_package_tools`):
    - Default policy query -> returns mode `"enforcing"`.
    - Package evaluation for `"curl"` -> returns `verdict.allowed = true`.
    - Package evaluation for `"telnet"` -> returns `ok = false`, `verdict.allowed = false`.
    - Tool discovery assertion confirms `aios.package.policy` is advertised in manifest.

## Runner Integration (PM7)
Added `test_pm7_security_policy` to `tools/test_package_suites.py`:
```python
def test_pm7_security_policy():
    return _run_cargo_test(
        ["--test", "test_package_policy"],
        "PM7",
        "package security policy evaluation & invariants (PP1..PP6)",
    )
```
Master test runner output:
```
[+] PM1 package data model integrity & invariants (PM1..PM5)
[+] PM2 package core service integrity & invariants (CS1..CS5)
[+] PM3 package CLI surface commands & options (validate/list/show/search/plan/apply)
[+] PM4 package MCP tool surface (validate/list/get/plan/search/apply)
[+] PM5 package configuration resolution & invariants (PC1..PC6)
[+] PM6 package automated integration test matrix (PT1..PT6)
[+] PM7 package security policy evaluation & invariants (PP1..PP6)

PASS: package_suites criteria (PM1..PM7)
```
