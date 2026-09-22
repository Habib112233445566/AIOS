# Task Evidence: T-02176 - PEP Decision Engine: Observability: Integration

## Task Metadata
- **Task ID**: `T-02176`
- **Sub-Epic**: Sub-Epic 8: Observability Subsystem
- **Component**: `aiosh-cli` and `aiosh-mcp` (Rust)
- **Date**: 2026-09-22
- **Status**: Completed

## 1. Summary of Changes
Integrated the PEP Observability Subsystem across both operational surfaces:

1. **CLI Surface (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
   - Added subcommand `aiosh pep report [--store <path>] [--json]` to `cmd_pep`.
   - Wired `PepObservabilityReport::generate` and `report.validate()`.
   - Emits structured audit row through `classify_and_emit(&mut ctx, "pep", "report", ...)`.
   - Supports structured JSON output (`--json`) and human-readable terminal output.
   - Updated main usage banner and `aiosh pep --help`.

2. **MCP Surface (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
   - Registered tool `aios.pep.report` in `tool_manifest` with input schema: `store_path`, `policy_path`, `grant_id`.
   - Implemented execution handler under `call_tool` routing through `dispatch::recorded_call`.
   - Validates path hygiene and security policy before report generation.

3. **Smoke Suites**:
   - `code/aiosh-cli/tests/test_pep_cli_smoke.py`: Added `test_pep_report_cli` asserting returncode 0 and valid report payload.
   - `code/aiosh-mcp/tests/test_pep_decision_smoke.py`: Updated `test_tool_registration` to verify `aios.pep.report` registration and added `test_pep_observability_report` calling the MCP tool.

## 2. Verification Results
- `python code/aiosh-cli/tests/test_pep_cli_smoke.py` -> 6/6 tests passed.
- `python code/aiosh-mcp/tests/test_pep_decision_smoke.py` -> 4/4 suites passed (tool registration, evaluation, persistent lifecycle, observability report).
