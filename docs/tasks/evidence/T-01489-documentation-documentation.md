# T-01489: User Session Bootstrap Documentation

**Date:** 2026-09-16  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** User Session Bootstrap / Documentation  
**Task ID:** T-01489  

---

## 1. Documentation Scope & Shipped Artifacts

This task documents the **User Session Bootstrap** subsystem for operators and autonomous agents, establishing production reference documentation, copy-pasteable invocation examples, constraints, and known limitations.

### Shipped Documentation Artifacts
1. **[`docs/user_session_bootstrap.md`](file:///docs/user_session_bootstrap.md)**:
   - Complete 9-section architectural and operational guide.
   - Formal specification of data models (`UserSessionSpec`, `UserSessionStatus`), lifecycle state machine (FSM), seat arbitration (`seat0`), configuration resolver, security policy engine, observability telemetry, CLI surface (`aiosh session *`), and MCP tool surface (`aios.session.*`).
   - Detailed specification of error envelopes and immutable audit logging.
   - Comprehensive constraints, security policies, and task evidence links.
2. **[`code/aiosh-mcp/README.md`](file:///code/aiosh-mcp/README.md)**:
   - Autonomous agent documentation for all 8 `aios.session.*` MCP tools.
   - Explicit documentation of PEP authorization requirements for state-mutating actions (`aios.session.create`, `aios.session.action`).
   - Copy-pasteable JSON-RPC 2.0 payloads with `grant_id`.

---

## 2. Invocation References & Copy-Pasteable Examples

### 2.1 Operator CLI Invocations (`aiosh session`)
```bash
# Validate a session specification
aiosh session validate '{"session_id":"sess-dev-01","username":"kali","uid":1000,"gid":1000,"session_type":"x11","session_class":"user","seat":"seat0","display":":0"}' --json

# Provision and register a session
aiosh session create '{"session_id":"sess-dev-01","username":"kali","uid":1000,"gid":1000,"session_type":"x11","session_class":"user","seat":"seat0","display":":0"}' --json

# List active sessions on seat0
aiosh session list --seat seat0 --json

# Advance session lifecycle
aiosh session authenticate sess-dev-01 --json
aiosh session activate sess-dev-01 --json
aiosh session lock sess-dev-01 --json
aiosh session unlock sess-dev-01 --json
aiosh session terminate sess-dev-01 --json

# Query telemetry and security policy compliance
aiosh session stats --json
aiosh session policy --json
```

### 2.2 Autonomous Agent MCP Tool Invocations (`aios.session.*`)

#### Step 1: Create a PEP Grant
```bash
aiosh grant create --to "agent:copilot" --tools "aios.session.*"
```

#### Step 2: Provision a New Session via JSON-RPC
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.session.create",
    "arguments": {
      "grant_id": "gr_eb284bdf79721da3",
      "spec": {
        "session_id": "sess-copilot-01",
        "username": "kali",
        "uid": 1000,
        "gid": 1000,
        "session_type": "ai_agent",
        "session_class": "agent",
        "seat": "seat0",
        "vtnr": null,
        "display": null,
        "remote_host": null,
        "environment": {
          "AIOS_AGENT": "1"
        }
      }
    }
  }
}
```

#### Step 3: Transition Session State via JSON-RPC
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.session.action",
    "arguments": {
      "session_id": "sess-copilot-01",
      "action": "lock",
      "grant_id": "gr_eb284bdf79721da3"
    }
  }
}
```

---

## 3. Constraints & Known Limitations

1. **Seat0 Mutual Exclusion**: At most one active session may hold `Foreground` status on `seat0` at any given time. Any new activation automatically demotes existing sessions on that seat to `Background` (`CS2`).
2. **Root Login Prohibitions**: Interactive login sessions running as UID 0 (`root`) are strictly forbidden by default security policy (`SSP1`). Privileged tasks must originate from unprivileged user accounts with audited privilege escalation.
3. **Remote Console Prohibitions**: Sessions specifying a `remote_host` are barred from attaching to local hardware seats (`seat0` or console VT seats) (`SSP3`).
4. **Environment Sanitization**: Dynamic linker overrides (`LD_PRELOAD`, `LD_AUDIT`, wildcard `LD_*`) and dangerous shell startup hooks (`BASH_ENV`, `ENV`, `PYTHONSTARTUP`, `PERL5LIB`, `RUBYOPT`, `PROMPT_COMMAND`, `GCC_EXEC_PREFIX`) are blocked. Underscore-prefixed variants (e.g. `_BASH_ENV`) are normalized and blocked (`SSP4`).
5. **Session Capacity Limits**: Maximum 32 concurrent sessions per user (`SSP6`) and 1,024 total tracked sessions across the system (`SSP7`).
6. **Atomic Persistence & File Permissions**: Store updates are written to a temporary sibling file with POSIX mode `0600` before atomic renaming, eliminating symlink hijacking and default-umask race conditions.
7. **PEP Gate Authorization**: MCP tools that alter state (`aios.session.create`, `aios.session.action`) require an unexpired PEP grant matching `aios.session.*`. Requests without a grant fail immediately with `gate: "pep"`.

---

## 4. Evidence Traceability & Acceptance

- [x] Docs updated with working copy-pasteable examples for CLI and MCP surfaces.
- [x] Constraints, invariants, and security limitations are stated honestly and completely.
- [x] All 9 required sections in `docs/user_session_bootstrap.md` maintained.
- [x] `python tools/test_session_doc.py` passes all D1..D6 criteria.
- [x] Task evidence files linked from documentation.
