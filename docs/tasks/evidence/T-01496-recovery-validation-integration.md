# T-01496: User Session Bootstrap Recovery & Validation Integration

**Date:** 2026-09-16  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** User Session Bootstrap / Recovery & Validation  
**Task ID:** T-01496  

---

## 1. Integration Summary

Task `T-01496` wires the **User Session Bootstrap Recovery & Validation** subsystem into its real production interfaces across both the CLI (`aiosh`) and MCP server (`aios.session.*`).

Both interfaces provide non-destructive validation and self-healing quarantine of damaged or corrupted session stores with complete telemetry and audit emission.

---

## 2. Integrated Production Surfaces

### 2.1 CLI Surface (`code/aiosh-rust/aiosh-cli/src/main.rs`)
- Added subcommands:
  - `aiosh session check [--fix] [--store <path>] [--json]`
  - `aiosh session recover [--store <path>] [--json]` (alias for `check --fix`)
- Provides human-readable validation summaries:
  - Healthy vs Unhealthy state
  - Total, valid, and invalid session counts
  - Itemized errors and seat arbitration warnings
  - Quarantined backup path display
- Provides JSON-RPC output (`--json`) with complete error codes and machine-readable `SessionValidationReport`.
- Audited via `classify_and_emit` to context audit ring under `session.check` / `session.repair`.

### 2.2 MCP Surface (`code/aiosh-rust/aiosh-mcp/src/main.rs`)
- Registered tool `aios.session.check` in MCP manifest (`tools/list`):
  ```json
  {
    "name": "aios.session.check",
    "description": "Validate on-disk user session store integrity and optionally perform non-destructive recovery (SSR1..SSR5)",
    "inputSchema": {
      "type": "object",
      "properties": {
        "store_path": { "type": "string", "description": "Optional custom path to the user session store JSON file" },
        "auto_recover": { "type": "boolean", "description": "Automatically repair corrupted or invalid session store with timestamped backup" },
        "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
      },
      "additionalProperties": false
    }
  }
  ```
- Handles `auto_recover: true`: loads or recovers corrupted store, preserves backup file, and returns updated healthy status.
- Dispatched through `dispatch::recorded_call` ensuring ring audit and PEP grant traceability.

### 2.3 Crate Root Re-Exports (`code/aiosh-rust/aiosh-core/src/lib.rs`)
- Re-exported core validation and recovery types:
  ```rust
  pub use session_recovery::{SessionRecoveryAction, SessionValidationReport};
  ```

---

## 3. Verification & Evidence

### 3.1 CLI Smoke Test
- Healthy store check:
  ```
  User Session Store Validation Report:
    Store Path:        /var/run/aios/sessions.json
    Status:            HEALTHY
    Total Sessions:    1
    Valid Sessions:    1
    Invalid Sessions:  0
  ```
- Corrupted store recovery:
  ```json
  {"code":0,"data":{"backup_path":"...\\temp.json.bak.20260916_145253_988893","errors":[],"evaluated_at":"2026-09-16T14:52:53.994041600+00:00","healthy":true,"invalid_sessions":0,"recovered":true,"store_path":"...\\temp.json","total_sessions":1,"valid_sessions":1,"warnings":[]},"error":null}
  ```

### 3.2 MCP In-Tree Tests
`cargo test -p aiosh-mcp`:
```
running 11 tests
test tests::test_mcp_session_validate_tools ... ok
...
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.08s
```

### 3.3 End-to-End JSON-RPC Smoke Test
`python code/aiosh-mcp/tests/test_session_mcp_smoke.py`:
```
=== RUNNING USER SESSION BOOTSTRAP MCP SMOKE TESTS ===
PASS: tools/list contains all 6 aios.session.* tools
PASS: aios.session.validate (valid, invalid, boundary, missing)
PASS: aios.session.list & aios.session.get (filtering, inspection, error modes)
PASS: aios.session.action (PEP enforcement, lock, unlock, unknown action, missing params)
PASS: aios.session.create & persistence (PEP enforcement, create, get, duplicate rejection, invalid spec)
PASS: Cross-surface CLI <-> MCP parity & state sharing
PASS: aios.session.check validation and quarantine recovery
PASS: MCP session hardening (payload limits, query bounds, ID injection, store path sanitization)

ALL USER SESSION BOOTSTRAP MCP SMOKE TESTS PASSED!
```

### 3.4 Master Session Test Suite
`python tools/test_session_suites.py`:
```
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP in-tree unit test suite
[+] SB3 session MCP tool surface JSON-RPC smoke test (aios.session.*)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)
[+] SB5 session configuration resolution, invariants & precedence (SC1..SC7)
[+] SB6 session automated integration test suite (SBT1..SBT5)
[+] SB7 session security policy enforcement & invariants (SSP1..SSP7)
[+] SB8 session observability & telemetry metrics (SSO1..SSO6)
[+] SB9 session documentation architecture & operational guide (D1..D6)

PASS: session_suites criteria (SB1..SB9)
```
