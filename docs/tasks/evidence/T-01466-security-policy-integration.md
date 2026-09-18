# T-01466: User Session Bootstrap — Security Policy: Integration

## Metadata
- **Task ID:** `T-01466`
- **Subsystem:** `tools/test_session_suites.py`, `code/aiosh-rust/aiosh-cli`, `code/aiosh-rust/aiosh-mcp`
- **Component:** User Session Bootstrap Security Policy Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Integration Deliverables

1. **Criterion `SB7` Integration in `tools/test_session_suites.py`**:
   - Added `test_sb7_security_policy` running `--test test_session_policy`.
   - Verified that `tools/test_session_suites.py` validates all criteria `SB1..SB7`.

2. **CLI Production Surface Integration**:
   - Subcommand `aiosh session policy [--policy <path>] [--spec <file_or_json>] [--store <path>] [--json]` exposed in production binary.
   - Verified SQLite WAL non-repudiation audit logging for each policy evaluation event.

3. **MCP Tool Surface Integration**:
   - Registered and exposed `aios.session.policy` in `list_tools`.
   - Gated with PEP capability authorization and SQLite WAL recording via `dispatch::recorded_call`.

## 2. Master Verification Output
```text
python tools/test_session_suites.py
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP in-tree unit test suite
[+] SB3 session MCP tool surface JSON-RPC smoke test (aios.session.*)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)
[+] SB5 session configuration resolution, invariants & precedence (SC1..SC7)
[+] SB6 session automated integration test suite (SBT1..SBT5)
[+] SB7 session security policy enforcement & invariants (SSP1..SSP7)

PASS: session_suites criteria (SB1..SB7)
```
