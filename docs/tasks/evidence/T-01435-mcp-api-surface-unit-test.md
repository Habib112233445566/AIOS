# T-01435: User Session Bootstrap - MCP/API Surface: Unit Test

## Metadata
- **Task ID:** `T-01435`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap MCP/API Surface Unit & Smoke Tests (`code/aiosh-mcp/tests/test_session_mcp_smoke.py`, `code/aiosh-rust/aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (5/10) — MCP/API Surface Unit Test

---

## 1. Test Suite Coverage & Design

Created standalone test suite `code/aiosh-mcp/tests/test_session_mcp_smoke.py` asserting JSON-RPC 2.0 protocol adherence over bidirectional stdio streams for all 5 `aios.session.*` MCP tools:

### 1.1 Tool Manifest Discovery (`test_manifest`)
- Queries `tools/list` over JSON-RPC stdio.
- Asserts that all 5 authoritative session tools are present in the returned manifest:
  - `aios.session.validate`
  - `aios.session.list`
  - `aios.session.get`
  - `aios.session.action`
  - `aios.session.create`

### 1.2 Validation Surface (`test_session_validate`)
- **Valid Session ID**: Validates identifier syntax (`sess-01`) -> returns `ok: true, valid: true`.
- **Invalid Session ID (Boundary/Negative)**: Tests path traversal (`../bad`) -> returns `ok: false`.
- **Valid Username**: Validates POSIX username (`kali`) -> returns `ok: true, valid: true`.
- **Invalid Username (Negative)**: Tests invalid characters (`Kali_Invalid!`) -> returns `ok: false`.
- **Valid Specification (Happy Path)**: Full `UserSessionSpec` with TTY/X11/Wayland/AiAgent parameters -> returns `ok: true, valid: true`.
- **Invalid Specification (Negative)**: X11 session missing display -> returns `ok: false`.
- **Missing Parameters**: Empty argument dictionary -> returns `ok: false`.

### 1.3 Discovery & Inspection Surface (`test_session_list_and_get`)
- **List Default Sessions**: Queries store; verifies presence of canonical `greeter-seat0` on `seat0`.
- **Filter by Seat**: Queries with `seat="seat0"`.
- **Filter by Non-Existent User**: Queries with `username="nonexistentuser"` -> returns `count: 0`.
- **Get Session**: Inspects `greeter-seat0` -> asserts username `lightdm`, state `active`.
- **Get Non-Existent Session (Negative)**: Queries `ghost-session` -> returns `ok: false`.
- **Missing Session ID**: Calls `aios.session.get` with empty arguments -> returns `ok: false`.

### 1.4 Lifecycle Transition Surface (`test_session_action`)
- **Stateful Lock & Unlock**:
  - Executes `action="lock"` on `greeter-seat0` with persistent store file -> asserts `new_state: "locked"`.
  - Executes `action="unlock"` on locked session -> asserts `new_state: "active"`.
- **Unknown Action (Negative)**: Calls `action="self_destruct"` -> returns `ok: false`.
- **Target Non-Existent Session (Negative)**: Attempts action on `ghost` -> returns `ok: false`.
- **Missing Action Parameter**: Returns `ok: false`.

### 1.5 Provisioning & Persistence Surface (`test_session_create_and_persistence`)
- **Create Session (Happy Path)**: Creates agent session `agent-copilot-smoke` (`session_type: "ai_agent"`, `session_class: "agent"`). Asserts initial state `initializing`.
- **State Persistence**: Re-reads newly created session via `aios.session.get` using custom `store_path`.
- **Duplicate Session Rejection (Negative)**: Attempts to recreate identical session ID -> returns `ok: false` with `"already exists"`.
- **Invalid Spec (Negative)**: Rejects spec with slash characters in `session_id` (`bad/id/with/slashes`).
- **Missing Spec (Negative)**: Rejects call lacking required `spec` parameter.

---

## 2. Test Execution Outputs

### 2.1 Standalone Python MCP Smoke Suite
```text
=== RUNNING USER SESSION BOOTSTRAP MCP SMOKE TESTS ===
PASS: tools/list contains all 5 aios.session.* tools
PASS: aios.session.validate (valid, invalid, boundary, missing)
PASS: aios.session.list & aios.session.get (filtering, inspection, error modes)
PASS: aios.session.action (lock, unlock, unknown action, missing params)
PASS: aios.session.create & persistence (create, get, duplicate rejection, invalid spec)

ALL USER SESSION BOOTSTRAP MCP SMOKE TESTS PASSED!
```

### 2.2 In-Tree Rust MCP Unit Test Suite (`cargo test -p aiosh-mcp --bin aiosh-mcp test_mcp_session_validate_tools`)
```text
running 1 test
test tests::test_mcp_session_validate_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.11s
```

### 2.3 Master Session Subsystem Suite (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP in-tree unit test suite
[+] SB3 session MCP tool surface JSON-RPC smoke test (aios.session.*)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

---

## 3. Acceptance Verification
- [x] New test file `code/aiosh-mcp/tests/test_session_mcp_smoke.py` created and runs standalone with 100% pass rate.
- [x] Comprehensive negative and boundary cases asserted across all 5 tools.
- [x] Observable behavior (JSON-RPC return envelopes, persistent file states, error strings) verified.
- [x] Integrated into master subsystem runner `tools/test_session_suites.py` under criterion `SB3`.
