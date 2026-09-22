# Task Evidence: T-02206 (Grant Lifecycle / data model: Integration)

## 1. Scope & Execution
Integrated the Grant Lifecycle Data Model (`pep_grant.rs`) across CLI and MCP operator/agent surfaces:
- **CLI (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
  - Implemented `aiosh pep grant list [--store <path>] [--subject <subject>] [--json]`: Lists registered grants with subject filtering.
  - Implemented `aiosh pep grant inspect <grant_id> [--store <path>] [--json]`: Retrieves full grant detail and lifecycle metadata.
  - Implemented `aiosh pep grant validate <grant_id> [--subject <subject>] [--right <right>] [--store <path>] [--json]`: Evaluates active state, temporal validity, quota limits, and subject/right authorization.
  - Implemented `aiosh pep grant revoke <grant_id> [--reason <reason>] [--cascade] [--store <path>] [--json]`: Revokes a grant with optional recursive cascading to all child grants.
- **MCP (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
  - Registered 4 tools in `tool_manifest()`: `aios.pep.grant.list`, `aios.pep.grant.inspect`, `aios.pep.grant.validate`, `aios.pep.grant.revoke`.
  - Implemented tool execution handlers in `call_tool()` routing through `dispatch::recorded_call`, writing an immutable row to SQLite audit ring for every call.
- **Integration Tests**:
  - Added `test_pep_grant_cli()` in `code/aiosh-cli/tests/test_pep_cli_smoke.py` (covering list, inspect, validate, revoke, and revoked rejection).
  - Added `test_pep_grant_mcp()` in `code/aiosh-mcp/tests/test_pep_decision_smoke.py` (covering MCP tool discovery, inspect, action validation, and cascade revocation).

## 2. Execution Results
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
PASS: aiosh pep grant CLI integration
=== All PEP CLI tests passed ===

> python code/aiosh-mcp/tests/test_pep_decision_smoke.py
=== PEP Decision Engine MCP Smoke Test ===
TEST: tool registration via tools/list ... OK
TEST: PEP decision evaluation ... OK
TEST: PEP MCP persistent lifecycle (status, rule_add, list, eval, remove) ... OK
TEST: PEP observability report ... OK
TEST: PEP MCP documentation tool (list, get, search) ... OK
TEST: PEP MCP recovery & validation tools (validate, recover) ... OK
TEST: PEP MCP grant lifecycle tools (list, inspect, validate, revoke) ... OK
=== All PEP Decision Engine smoke tests passed ===
```

## 3. Acceptance Confirmation
- [x] Feature reachable through production CLI and MCP tool surfaces.
- [x] Integration smoke tests pass end-to-end (100% pass rate).
- [x] Cross-substrate canonical JSON schema parity verified.
