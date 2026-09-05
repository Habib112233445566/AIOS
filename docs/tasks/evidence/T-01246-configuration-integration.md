# T-01246: Package Management - Configuration: Integration

## Metadata
- **Task ID:** `T-01246`
- **Subsystem:** `code/aiosh-rust`
- **Component:** Package Management Configuration Subsystem Integration
- **Status:** Complete

## 1. Integration Scope
Integrated `aiosh_core::package_config::PackageConfig` into both operator and autonomous agent user surfaces:
1. **Operator CLI (`aiosh package config`)**:
   - Wired in `code/aiosh-rust/aiosh-cli/src/main.rs`.
   - Supports `--config <path>` flag and `--json` formatting.
   - Enforces ADR-0035 classification and audit logging via `classify_and_emit`.
   - Returns structured JSON envelope `{ "code": 0, "data": ..., "error": null }`.
2. **Autonomous Agent MCP Tool (`aios.package.config`)**:
   - Registered in `Server::tool_manifest()` in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
   - Handled in `Server::call_tool()` via `dispatch::recorded_call`, enforcing Gate #1 (classifier) and Gate #2 (PEP).
   - Writes immutable audit records with SHA-256 chain extensions.
3. **Master Test Runner Integration (`tools/test_package_suites.py`)**:
   - Added criterion `PM5`: `package configuration resolution & invariants (PC1..PC6)` executing `--test test_package_config`.

## 2. End-to-End Verification
- Direct execution of `aiosh package config --json`:
  ```json
  {
    "code": 0,
    "data": {
      "allowed_repositories": [
        "https://deb.debian.org/debian",
        "https://dl-cdn.alpinelinux.org/alpine/v3.19/main"
      ],
      "auto_persist": false,
      "default_format": "deb",
      "max_entity_count": 10000,
      "max_store_size_bytes": 10485760,
      "store_path": ".aios/packages.json"
    },
    "error": null
  }
  ```
- MCP tool execution verified in `test_mcp_package_tools`.
