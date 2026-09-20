# T-02126: CLI Surface Integration — PEP Decision Engine

## Overview
- **Task ID**: `T-02126`
- **Sub-Epic**: 3 (CLI Surface)
- **Component**: PEP Decision CLI Surface (`aiosh pep`)
- **Status**: Completed

## Integration Details
1. **Binary Wiring & Discovery**:
   - `aiosh pep` wired directly into `code/aiosh-rust/aiosh-cli/src/main.rs`.
   - Discoverable via root `aiosh --help` overview and subcommand `aiosh pep --help`.
2. **Substrate Parity & Persistence**:
   - Shared canonical JSON store backed by `PepDecisionService::save_to_path` and `load_or_recover`.
   - Every operation integrates with the AIOS SQLite `AuditRing`, emitting audit records with classification and outcome details.
3. **End-to-End Smoke Test**:
   - Executed `code/aiosh-cli/tests/test_pep_cli_smoke.py`.
   - Tests help, unknown subcommand, path hygiene, rule lifecycle, and authorization evaluations.
   - Result: PASS.
