# T-01434: User Session Bootstrap - MCP/API Surface: Implementation

## Metadata
- **Task ID:** `T-01434`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap MCP/API Surface Implementation (`code/aiosh-rust/aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (4/10) — MCP/API Surface Implementation

---

## 1. Implementation Summary

Implemented the working behavior for `aios.session.create` within `code/aiosh-rust/aiosh-mcp/src/main.rs`:
1. **Tool Schema (`tool_manifest`)**:
   - Registered `aios.session.create` accepting `spec` (object), optional `store_path` (string), and optional `grant_id` (string).
2. **Dispatch Handler (`call_tool`)**:
   - Ingests `spec` as either a parsed JSON object or serialized JSON string with 1 MiB payload ceiling enforcement.
   - Validates specification against formal invariants via `aiosh_core::session::validate_user_session_spec`.
   - Validates `store_path` length ($\le 1,024$) and rejects ASCII control characters.
   - Delegates session instantiation to `aiosh_core::session_service::UserSessionService::create_session`.
   - Performs atomic persistence to designated `store_path` if provided.
   - Returns structured envelope containing `session_id`, `report` (`UserSessionActionReport`), `spec`, and current `status`.
   - Emits append-only SHA-256 hash-chained audit event via `dispatch::recorded_call` into `$AIOSH_HOME/audit.db`.

---

## 2. Unit Test Verification

Extended `tests::test_mcp_session_validate_tools` in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
- Verified discovery of `aios.session.create` in manifest.
- Verified successful session creation for `agent-copilot-01` (`session_type: "ai_agent"`, `session_class: "agent"`).
- Verified duplicate session ID rejection (`greeter-seat0`).
- Verified rejection of invalid specifications (e.g. path traversal `../evil`).
- Verified rejection of invocations missing required `spec` parameter.

### 2.1 Unit Test Output (`cargo test -p aiosh-mcp --bin aiosh-mcp test_mcp_session_validate_tools`)
```text
running 1 test
test tests::test_mcp_session_validate_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.11s
```

### 2.2 Master Test Runner Output (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

---

## 3. Acceptance Verification
- [x] Targeted test passes with zero failures.
- [x] No regressions in existing session test suites (`SB1..SB4`).
- [x] Consequential mutations write to audit ring via `dispatch::recorded_call`.
