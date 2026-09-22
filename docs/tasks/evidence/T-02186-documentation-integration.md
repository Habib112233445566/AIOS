# Task Evidence: T-02186 (documentation: Integration)

## 1. Objective & Scope
Integrate the PEP Decision Engine documentation subsystem (`PepDocIndex`) into its real production surfaces across:
1. **AIOS CLI Substrate**: `aiosh pep doc [list | show <topic_id> | search <query>] [--json]`
2. **MCP JSON-RPC Server**: `aios.pep.doc` tool exposing `action: list | get | search`, `topic_id`, `query`, `category`
3. **Audit Ring Invariant**: Consequential calls and queries route through `dispatch::recorded_call` and `classify_and_emit` to preserve full audit traceability in the SQLite ring buffer.

## 2. Changes Implemented
- **CLI Substrate (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
  - Integrated `cmd_pep` subcommand `doc` supporting `list`, `show <topic_id>`, and `search <query>`.
  - Added `--json` structured envelope output option with category filtering and human-readable Markdown rendering.
  - Wired into root CLI dispatch and usage banners (`aiosh --help` and `aiosh pep --help`).
- **MCP Substrate (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
  - Registered `aios.pep.doc` in `tool_manifest` with strict schema validation.
  - Implemented execution handler routing through `dispatch::recorded_call`.
- **Smoke Integration Test Suites**:
  - `code/aiosh-cli/tests/test_pep_cli_smoke.py`: Added `test_pep_doc_cli()` asserting `doc list`, `doc show pep-arch`, and `doc search DenyOverrides`.
  - `code/aiosh-mcp/tests/test_pep_decision_smoke.py`: Added `test_pep_doc_mcp()` asserting tool registration, listing canonical topics, fetching `pep-arch`, and query search.

## 3. Verification & Evidence
Both integration smoke suites executed and verified:
```
$ python code/aiosh-cli/tests/test_pep_cli_smoke.py
PASS: aiosh pep --help
PASS: aiosh pep unknown_cmd returns 2
PASS: aiosh pep path hygiene enforcement
PASS: aiosh pep lifecycle and evaluation
PASS: aiosh pep security policy privilege boundary
PASS: aiosh pep report CLI integration
PASS: aiosh pep doc CLI integration
=== All PEP CLI tests passed ===

$ python code/aiosh-mcp/tests/test_pep_decision_smoke.py
=== PEP Decision Engine MCP Smoke Test ===
TEST: tool registration via tools/list ... OK
TEST: PEP decision evaluation ... OK
TEST: PEP MCP persistent lifecycle (status, rule_add, list, eval, remove) ... OK
TEST: PEP observability report ... OK
TEST: PEP MCP documentation tool (list, get, search) ... OK
=== All PEP Decision Engine smoke tests passed ===
```

## 4. Acceptance Confirmation
- [x] Feature reachable through production surfaces (`aiosh pep doc` and `aios.pep.doc`).
- [x] Integration smoke passes end-to-end for both CLI and MCP.
- [x] Audit invariants maintained via SQLite event ring integration.
