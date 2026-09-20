# Task Evidence: T-02156 (PEP Decision Engine Automated Tests: Integration)

## Overview
- **Task ID**: `T-02156`
- **Task Name**: automated tests: Integration
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / PEP Decision Engine
- **Sub-Epic**: Sub-Epic 6: Automated Tests Subsystem
- **Timestamp**: 2026-09-21T01:17:50+05:00
- **Status**: COMPLETED

## Integration Summary

### 1. Production Surface Integration
- The PEP Decision Engine automated test harness (`test_pep_decision_e2e.rs`) is integrated into the core cargo workspace test execution pipeline.
- Exercises the end-to-end call path across:
  - `aiosh_core::pep_decision`: Core decision evaluation, combining algorithms (`DenyOverrides`, `PermitOverrides`, `FirstApplicable`), and obligation handling.
  - `aiosh_core::pep_decision_service`: Multi-rule store management, capacity bounds enforcement (5,000 rules), indexed rule evaluation, and non-destructive quarantine (`.bak.<timestamp>`).
  - `aiosh_core::pep_config`: Environment variable ingestion (`AIOSH_PEP_*`) and atomic config persistence.
  - `aiosh-cli`: Command execution via `cmd_pep` (`evaluate`, `rule-add`, `rule-list`, `rule-remove`, `status`).
  - `aiosh-mcp`: JSON-RPC MCP tools (`aios.pep.status`, `aios.pep.rule_add`, `aios.pep.rule_list`, `aios.pep.rule_remove`, `aios.pep.evaluate`).

### 2. Cross-Substrate Parity
- Verified identical policy evaluation outcomes across Rust native execution, CLI invocations, and MCP tool calls.
- Canonical JSON persistence format is shared seamlessly between `aiosh-core`, `aiosh-cli`, and `aiosh-mcp`.

### 3. Verification Commands & Outputs
```bash
python code/aiosh-cli/tests/test_pep_cli_smoke.py
python code/aiosh-cli/tests/test_pep_config_smoke.py
python code/aiosh-mcp/tests/test_pep_decision_smoke.py
```
Output:
- `PASS: aiosh pep --help`
- `PASS: aiosh pep unknown_cmd returns 2`
- `PASS: aiosh pep path hygiene enforcement`
- `PASS: aiosh pep lifecycle and evaluation`
- `TEST: AIOSH_PEP_STORE_PATH environment variable override ... OK`
- `TEST: AIOSH_PEP_CONFIG file loading ... OK`
- `TEST: Invalid env store path hygiene rejection ... OK`
- `TEST: tool registration via tools/list ... OK`
- `TEST: PEP decision evaluation ... OK`
- `TEST: PEP MCP persistent lifecycle (status, rule_add, list, eval, remove) ... OK`
- All tests passing with exit code 0.
