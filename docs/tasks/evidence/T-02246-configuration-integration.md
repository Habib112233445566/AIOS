# Task Evidence: T-02246 (Grant Lifecycle Configuration: Integration)

## Overview
- **Task ID**: `T-02246`
- **Task Name**: Grant Lifecycle Configuration: Integration
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Grant Lifecycle
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Timestamp**: 2026-09-23T01:27:00+05:00
- **Status**: COMPLETED

## Implementation & Integration Summary

1. **CLI Surface Integration (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
   - Integrated `PepGrantConfig::from_env()` in `cmd_pep` for all grant subcommands (`issue`, `list`, `inspect`, `validate`, `revoke`, `sweep`).
   - Default store path fallback now resolves to `grant_cfg.store_path`.
   - File size ceiling dynamically evaluates against `grant_cfg.max_store_bytes`.
   - `aiosh pep grant issue` defaults delegation depth to `grant_cfg.default_max_delegation_depth` when `--max-depth` is unspecified.

2. **MCP Surface Integration (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
   - Integrated `PepGrantConfig::from_env()` in `validate_and_open_grant_store` and `validate_and_open_grant_service`.
   - Default store path resolution honors `AIOSH_PEP_GRANT_CONFIG` and `AIOSH_PEP_GRANT_STORE_PATH` / `AIOSH_PEP_GRANT_STORE`.
   - File size ceiling dynamically evaluated against `grant_cfg.max_store_bytes`.
   - Automated sweep on load executed when `grant_cfg.auto_sweep_on_load` is active.

3. **Verification**:
   - CLI grant test suite `test_pep_grant_cli.py`: 5/5 test suites passed.
   - MCP grant test suite `test_pep_grant_mcp.py`: 100% passed.
   - Workspace compilation: `cargo check --workspace` clean with 0 warnings.
