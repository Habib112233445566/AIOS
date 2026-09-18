# T-01499: User Session Bootstrap Recovery & Validation Documentation

**Date:** 2026-09-16  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** User Session Bootstrap / Recovery & Validation  
**Task ID:** T-01499  

---

## 1. Documentation Scope

Task `T-01499` delivers comprehensive operational documentation for the **User Session Bootstrap Recovery & Validation** subsystem in:
1. Primary subsystem guide: [`docs/user_session_bootstrap.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/user_session_bootstrap.md) (§10)
2. MCP server reference: [`code/aiosh-mcp/README.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-mcp/README.md)

---

## 2. Content & Working Examples Documented

### 2.1 Invariant Architecture
- Documents invariants `SSR1..SSR5`:
  - `SSR1`: `valid_sessions + invalid_sessions == total_sessions`
  - `SSR2`: `healthy == (errors.is_empty() && invalid_sessions == 0)`
  - `SSR3`: `invalid_sessions > 0 => errors.len() >= invalid_sessions`
  - `SSR4`: Non-destructive quarantine via `.bak.<YYYYMMDD_HHMMSS_micros>` with POSIX `0600` permissions.
  - `SSR5`: Hardware seat foreground mutual exclusion and leader PID collision detection.

### 2.2 CLI Usage & Copy-Pasteable Commands
- Validation:
  ```bash
  aiosh session check --store /var/run/aios/sessions.json
  aiosh session check --store /var/run/aios/sessions.json --json
  ```
- Automated Recovery:
  ```bash
  aiosh session check --fix --store /var/run/aios/sessions.json --json
  aiosh session recover --store /var/run/aios/sessions.json --json
  ```

### 2.3 MCP Tool Invocations
- Validation via `aios.session.check`:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 8,
    "method": "tools/call",
    "params": {
      "name": "aios.session.check",
      "arguments": {
        "store_path": "/var/run/aios/sessions.json",
        "auto_recover": false
      }
    }
  }
  ```
- Automated recovery via `aios.session.check`:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 9,
    "method": "tools/call",
    "params": {
      "name": "aios.session.check",
      "arguments": {
        "store_path": "/var/run/aios/sessions.json",
        "auto_recover": true
      }
    }
  }
  ```

### 2.4 Constraints & Known Limitations
- POSIX `0600` permissions on quarantined backups.
- Seeded canonical reconstitution pre-seeded with `greeter-seat0`.
- 10,000 maximum collision attempts in backup path generator.
- 10,000 session capacity threshold (`MAX_STORE_CAPACITY`).

---

## 3. Evidence Verification

`python tools/test_session_doc.py`:
```
[+] D1 doc existence and size bounds (26647 bytes)
[+] D2 all 9 required sections present
[+] D3 zero forbidden placeholders/markers
[+] D4 policy invariants, CLI commands, and MCP tools coverage complete
[+] D5 negative rejection assertions verified
[+] D6 zero volatile snapshot counts (C6 compliant)

PASS: test_session_doc criteria (D1..D6)
```
Passes all documentation quality criteria D1..D6.
