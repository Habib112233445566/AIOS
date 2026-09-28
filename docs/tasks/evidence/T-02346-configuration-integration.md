# Task Evidence: T-02346 (Audit Chain Extensions / configuration: Integration)

## 1. Metadata
- **Task ID:** `T-02346`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions Configuration Integration
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 5: Configuration (6/10) — Integration

---

## 2. Integration Overview & Multi-Substrate Wire-Up

1. **CLI Surface Integration (`aiosh-cli`)**:
   - Integrated `AuditChainConfig::from_env()` into all `aiosh audit` commands (`query`, `inspect`, `ancestry`, `sign-verify`).
   - Added `aiosh audit config [--json]` subcommand to display current active configuration in table format or JSON.
   - Updated top-level `--help` usage documentation.

2. **MCP / JSON-RPC Surface Integration (`aiosh-mcp`)**:
   - Added `aios.audit.config` to `tools/list` schema with optional `config_path` argument.
   - Implemented `aios.audit.config` handler wrapped in `dispatch::recorded_call`, returning verified configuration parameters.

3. **Cross-Substrate Parity**:
   - Validated that default and environment overrides are consumed identically across Rust core, CLI commands, and MCP tools.

---

## 3. Verification Output

### 3.1 CLI Output
```text
> aiosh.exe audit config
=== Audit Chain Configuration ===
Version:                    1.0.0
DB Path:                    .aios/audit.db
Max Query Limit:            50
Default Lineage Depth:      16
Max Causal Links:           16
Max Extensions Bytes:       65536
Verify Signatures on Read:  true
Strict Provenance:          false
```

### 3.2 MCP Output
```text
> python code/aiosh-mcp/tests/test_audit_chain_mcp.py
Running test_mcp_audit_chain_tool_registration...
Running test_mcp_audit_query_filtering_and_bounds...
Running test_mcp_audit_inspect_valid_and_missing...
Running test_mcp_audit_ancestry_and_sign_verify...
Running test_mcp_audit_config...
=== All Audit Chain MCP Unit Tests Passed Successfully ===
```

---

## 4. Acceptance Confirmation
- [x] Configuration integrated across CLI and MCP surfaces.
- [x] Integration smoke tests pass end-to-end.
- [x] Cross-substrate parity validated.
