# Task Evidence: T-02146 (PEP Decision Engine Configuration: Integration)

## Overview
- **Task ID**: `T-02146`
- **Task Name**: configuration: Integration
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Timestamp**: 2026-09-21T01:03:35+05:00
- **Status**: COMPLETED

## Objective
Integrate `PepConfig` into the production execution paths of the PEP Decision Engine:
1. Wire `PepConfig::from_env()` into `cmd_pep` in `aiosh-cli` for dynamic store path resolution.
2. Confirm configuration precedence: CLI flag `--store` > Environment variables (`AIOSH_PEP_STORE_PATH`, `AIOSH_PEP_CONFIG`) > Defaults.
3. Validate with end-to-end integration smoke test `code/aiosh-cli/tests/test_pep_config_smoke.py`.

## Integration Details
- **CLI Integration**:
  - `cmd_pep` in `code/aiosh-rust/aiosh-cli/src/main.rs` updated to query `aiosh_core::PepConfig::from_env()` when `--store` is not explicitly provided on the command line.
  - Rejection of invalid store paths via `validate_pep_service_path` is preserved with exit code 2.
- **Automated Smoke Test**:
  - `code/aiosh-cli/tests/test_pep_config_smoke.py` tests:
    - Environment store path override (`AIOSH_PEP_STORE_PATH`).
    - Config file loading via `AIOSH_PEP_CONFIG`.
    - Path hygiene rejection on directory traversal attempt.
