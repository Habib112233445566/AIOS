# Task Evidence: T-02196 (recovery & validation: Integration)

## 1. Scope & Execution
Integrated the PEP Decision Engine Recovery & Validation subsystem across CLI and MCP operational surfaces:
- **CLI (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
  - Wired `aiosh pep validate <store_path>` to validate policy rule store files against structural and semantic invariants.
  - Wired `aiosh pep recover <store_path> [--strategy <salvage|quarantine|fail_closed>] [--backup] [--dry-run]` to perform resilient repair or quarantine.
- **MCP Server (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
  - Registered `aios.pep.validate` tool in server capabilities and tool manifest.
  - Registered `aios.pep.recover` tool in server capabilities and tool manifest.
  - Connected MCP tool dispatch with input sanitization and structured JSON response formatting.
- **Cross-Substrate Parity**:
  - Support for both JSON array (`rules: [...]`) and JSON object map (`rules: { "id": {...} }`) formats.
  - Output contracts unified across CLI exit codes and MCP tool JSON payloads.

## 2. Integration Verification
- Extended `code/aiosh-cli/tests/test_pep_cli_smoke.py` with `test_pep_recovery_cli()` covering:
  - Valid store inspection (`aiosh pep validate`).
  - Corrupt store detection and error reporting.
  - Policy store salvage recovery (`aiosh pep recover --strategy salvage --backup`).
  - Strict fail-closed recovery mode (`aiosh pep recover --strategy fail_closed`).
- Extended `code/aiosh-mcp/tests/test_pep_decision_smoke.py` with `test_pep_recovery_mcp()` covering:
  - Tool discovery via MCP `tools/list`.
  - RPC calls to `aios.pep.validate` for valid and corrupted files.
  - RPC calls to `aios.pep.recover` with dry-run and live recovery strategies.

## 3. Execution Results
```
> python code/aiosh-cli/tests/test_pep_cli_smoke.py
PASS: aiosh pep --help
PASS: aiosh pep unknown_cmd returns 2
PASS: aiosh pep path hygiene enforcement
PASS: aiosh pep lifecycle and evaluation
PASS: aiosh pep security policy privilege boundary
PASS: aiosh pep report CLI integration
PASS: aiosh pep doc CLI integration
PASS: aiosh pep recovery & validation CLI integration
=== All PEP CLI tests passed ===

> python code/aiosh-mcp/tests/test_pep_decision_smoke.py
=== PEP Decision Engine MCP Smoke Test ===
TEST: tool registration via tools/list ... OK
TEST: PEP decision evaluation ... OK
TEST: PEP MCP persistent lifecycle (status, rule_add, list, eval, remove) ... OK
TEST: PEP observability report ... OK
TEST: PEP MCP documentation tool (list, get, search) ... OK
TEST: PEP MCP recovery & validation tools (validate, recover) ... OK
=== All PEP Decision Engine smoke tests passed ===
```

## 4. Acceptance Confirmation
- [x] Feature reachable and functional through production CLI (`aiosh pep validate`, `aiosh pep recover`).
- [x] Feature discoverable and operational via MCP tools (`aios.pep.validate`, `aios.pep.recover`).
- [x] Cross-substrate parity validated across JSON array and map representations.
- [x] Integration smoke tests pass end-to-end without errors.
