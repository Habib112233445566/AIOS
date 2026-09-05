# T-01236: Package Management - MCP/API Surface: Integration

## Metadata
- **Task ID:** `T-01236`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Package Management MCP/API Surface Integration
- **Status:** Complete

## 1. Integrated Production Call Paths
The Package Management MCP surface has been fully integrated into the `aiosh-mcp` JSON-RPC server:
- **Server Registration (`Server::tool_manifest`)**:
  - `aios.package.validate`: Package name and spec validation (PM1..PM5).
  - `aios.package.list`: Store querying and enumeration with format/state/pattern/limit filtering.
  - `aios.package.get`: Detailed package specification lookup.
  - `aios.package.plan`: Deterministic transaction planning with dependency closure checks.
  - `aios.package.search`: Substring search over package names and descriptions.
  - `aios.package.apply`: Transaction application with dry-run support and optional disk persistence.
- **Execution Dispatch (`Server::call_tool`)**:
  - All 6 tools route through `dispatch::recorded_call`, verifying PEP capability authorization and emitting immutable audit rows to `audit.db`.

## 2. Cross-Substrate Parity
- **CLI & MCP Equivalence**: Every operation on `aiosh package` has a 1-to-1 equivalent MCP tool on `aiosh-mcp` using identical underlying domain logic from `aiosh-core::package` and `aiosh-core::package_service`.
- **Audit Logging (ADR-0035)**: Both CLI and MCP invocations write consistent audit entries into SQLite WAL `audit.db`.
- **Disk Persistence Format**: Shared JSON schema for on-disk package stores (`PackageStore::save_to_path` and `PackageStore::load_from_path`).

## 3. Automated Test Runner Suite Integration
Added criterion **`PM4`** to `tools/test_package_suites.py`:
- `PM4`: `package MCP tool surface (validate/list/get/plan/search/apply)` invoking `cargo test --bin aiosh-mcp test_mcp_package_tools`.

Runner output:
```text
[+] PM1 package data model integrity & invariants (PM1..PM5)
[+] PM2 package core service integrity & invariants (CS1..CS5)
[+] PM3 package CLI surface commands & options (validate/list/show/search/plan/apply)
[+] PM4 package MCP tool surface (validate/list/get/plan/search/apply)

PASS: package_suites criteria (PM1..PM4)
```
