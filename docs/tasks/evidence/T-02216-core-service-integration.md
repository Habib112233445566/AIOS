# Task Evidence: T-02216 (Grant Lifecycle / core service: Integration)

## 1. Scope & Execution
Integrated `PepGrantService` into both the CLI (`aiosh-cli`) and MCP server (`aiosh-mcp`) runtime layers:
1. **MCP Surface Integration**:
   - Registered `aios.pep.grant.attenuate` in `Server::tool_manifest` with JSON schema requiring `parent_id`, `child_id`, `child_subject`, and `rights`.
   - Registered `aios.pep.grant.sweep` in `Server::tool_manifest` with optional `store_path` parameter.
   - Wired dispatch in `Server::call_tool` executing through `dispatch::recorded_call` ensuring tamper-evident SQLite audit logging.
   - Verified that `attenuate` enforces `delegate` right containment and delegation depth decrement.
   - Verified that `sweep` evaluates grant expiration and quota exhaustion, updating storage atomically.
2. **CLI Surface Integration**:
   - Added `aiosh pep grant sweep [--store <PATH>] [--json]` to `cmd_pep` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
   - Wired command through `classify_and_emit` security audit telemetry and structured JSON envelopes.
3. **Automated Smoke & Integration Testing**:
   - Extended `code/aiosh-mcp/tests/test_pep_decision_smoke.py` asserting `aios.pep.grant.attenuate` and `aios.pep.grant.sweep` tool registration, execution, and hierarchical cascade revocation.
   - Extended `code/aiosh-cli/tests/test_pep_cli_smoke.py` asserting `aiosh pep grant sweep` invocation, exit code 0, and JSON output formatting.

---

## 2. Test Execution Output
```
> python -m pytest code/aiosh-cli/tests/test_pep_cli_smoke.py code/aiosh-mcp/tests/test_pep_decision_smoke.py
============================= test session starts =============================
platform win32 -- Python 3.14.6, pytest-9.1.1, pluggy-1.6.0
collected 16 items

code\aiosh-mcp .........                                                 [ 56%]
code\aiosh-mcp\tests\test_pep_decision_smoke.py .......                  [100%]

============================= 16 passed in 3.16s ==============================

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
```

---

## 3. Acceptance Confirmation
- [x] `PepGrantService` successfully integrated into `aiosh-mcp` with `attenuate` and `sweep` tools.
- [x] `PepGrantService` integrated into `aiosh-cli` with `pep grant sweep` command.
- [x] All MCP and CLI operations recorded via audit logging.
- [x] Integration tests passing 100% across MCP and CLI smoke suites.
