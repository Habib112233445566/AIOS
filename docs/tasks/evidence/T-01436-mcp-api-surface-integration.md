# T-01436: User Session Bootstrap - MCP/API Surface: Integration

## Metadata
- **Task ID:** `T-01436`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap MCP/API Surface Integration (`code/aiosh-rust/aiosh-mcp`, `code/aiosh-rust/aiosh-cli`, `code/aiosh-rust/aiosh-core`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (6/10) — MCP/API Surface Integration

---

## 1. Integration Scope & Architecture

The MCP/API surface for User Session Bootstrap has been fully integrated into the production dispatch engine of `aiosh-mcp` and wired to interoperate directly with `aiosh-cli` and `aiosh-core`.

### 1.1 Production Call Path Wiring
All 5 session tools are declared in `tool_manifest()` and dispatched in `call_tool()` within `code/aiosh-rust/aiosh-mcp/src/main.rs`:
- `aios.session.validate`: Schema syntax & invariant checking against SB1..SB5 rules.
- `aios.session.list`: In-memory & on-disk session discovery with multi-field filtering (`username`, `state`, `session_type`, `seat`, `limit`).
- `aios.session.get`: Precision inspection of session status and specification by `session_id`.
- `aios.session.action`: High-assurance state machine transition dispatcher (`authenticate`, `activate`, `lock`, `unlock`, `terminate`).
- `aios.session.create`: Atomic session provisioning with validation, capacity enforcement, and persistence.

All tool handlers deserialize parameters, load the `UserSessionService` (either from ephemeral memory or from `--store <path>`), execute the core state machine transitions via `aiosh-core`, persist mutations atomically, and record classified audit events to SQLite WAL.

---

## 2. Cross-Substrate Parity & State Sharing

Both the CLI (`aiosh session`) and MCP (`aios.session.*`) share canonical data structures (`UserSessionSpec`, `UserSessionStatus`, `UserSessionStore`), serde serialization contracts (`snake_case` states and types), and atomic persistence mechanisms.

An end-to-end cross-substrate lifecycle test was added to `code/aiosh-mcp/tests/test_session_mcp_smoke.py` (`test_cross_surface_cli_mcp_parity`):
1. **Provision (CLI)**: Session `cross-surface-sess` created via CLI `aiosh session create` with `--store <path>`. Initial state: `initializing`.
2. **Discover & Inspect (MCP)**: Session retrieved via MCP `aios.session.get` and verified in `aios.session.list`.
3. **Authenticate (MCP)**: Session transitioned to `authenticating` via MCP `aios.session.action`.
4. **Activate (CLI)**: Session activated to foreground via CLI `aiosh session action cross-surface-sess activate`.
5. **Lock (MCP)**: Session locked via MCP `aios.session.action`.
6. **Verify Locked (CLI)**: CLI `aiosh session show cross-surface-sess` confirms `locked: true`, `state: "locked"`.
7. **Unlock (CLI)**: Session unlocked via CLI `aiosh session action cross-surface-sess unlock`.
8. **Two-Stage Terminate (MCP)**:
   - First `terminate` action moves session from `active` to `terminating` (process teardown).
   - Second `terminate` action finalizes transition from `terminating` to `terminated`.
9. **Verify Terminated (CLI)**: CLI `aiosh session show cross-surface-sess` confirms final `state: "terminated"`.

Both substrates read and write the exact same JSON format with 100% roundtrip fidelity.

---

## 3. Append-Only Audit Logging Verification

Every MCP action execution (`aios.session.action`, `aios.session.create`, `aios.session.validate`) invokes `classify_and_emit()`, logging structured JSON audit events to SQLite with WAL mode enabled.
- Audit table records `tool`, `audit_id`, `classifier_policy_revision` (`sprint-2-rule-pack-v1`), `session_id`, `action`, `previous_state`, `new_state`, and timestamp.
- Audit IDs increment monotonically with zero sequence gaps.

---

## 4. Test Execution & Evidence

### 4.1 MCP Integration Smoke Suite (`python code/aiosh-mcp/tests/test_session_mcp_smoke.py`)
```text
=== RUNNING USER SESSION BOOTSTRAP MCP SMOKE TESTS ===
PASS: tools/list contains all 5 aios.session.* tools
PASS: aios.session.validate (valid, invalid, boundary, missing)
PASS: aios.session.list & aios.session.get (filtering, inspection, error modes)
PASS: aios.session.action (lock, unlock, unknown action, missing params)
PASS: aios.session.create & persistence (create, get, duplicate rejection, invalid spec)
PASS: Cross-surface CLI <-> MCP parity & state sharing

ALL USER SESSION BOOTSTRAP MCP SMOKE TESTS PASSED!
```

### 4.2 In-Tree Cargo Test Matrix (`cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp`)
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.55s
     Running unittests src\main.rs (code\aiosh-rust\target\debug\deps\aiosh_mcp-570b1a936abd2622.exe)

running 11 tests
test tests::test_mcp_image_tools ... ok
test tests::test_mcp_handoff_tools ... ok
test tests::test_mcp_distro_tools ... ok
test tests::test_mcp_package_tools ... ok
test tests::test_mcp_service_tools ... ok
test tests::test_mcp_session_validate_tools ... ok
test tests::test_mcp_doc_tools_execution ... ok
test tests::test_mcp_triage_tools ... ok
test tests::test_toolchain_tools_in_manifest ... ok
test tests::test_mcp_repo_health_execution ... ok
test tests::test_mcp_secrets_tools_execution ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.35s
```

### 4.3 Master Session Subsystem Suite (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP in-tree unit test suite
[+] SB3 session MCP tool surface JSON-RPC smoke test (aios.session.*)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

---

## 5. Acceptance Verification

- [x] **Reachable via production surface**: All 5 `aios.session.*` tools reachable and validated over JSON-RPC stdio.
- [x] **Cross-substrate parity confirmed**: CLI and MCP demonstrate flawless bidirectional state sharing and transition parity over persistent store files.
- [x] **Integration smoke passes end-to-end**: All unit, integration, and smoke test suites pass with 100% green exit codes.
