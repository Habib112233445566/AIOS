# T-01469: User Session Bootstrap — Security Policy: Documentation

## Metadata
- **Task ID:** `T-01469`
- **Subsystem:** `code/aiosh-cli`, `code/aiosh-mcp`, `code/aiosh-rust/aiosh-core`
- **Component:** User Session Bootstrap Security Policy Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Documentation Deliverables

### 1. CLI Commands (`code/aiosh-cli/README.md`)
Added full documentation for `aiosh session policy`:
```bash
# Evaluate active session store against default security policy
aiosh session policy

# JSON output
aiosh session policy --json

# Evaluate single session spec against default policy
aiosh session policy --spec /path/to/spec.json

# Evaluate with custom policy against custom session store
aiosh session policy --policy /etc/aios/session_policy.json --store /var/lib/aios/sessions.json
```

### 2. MCP Tools (`code/aiosh-mcp/README.md`)
Added tool registration and JSON-RPC 2.0 schema for `aios.session.policy`:
- **Name:** `aios.session.policy`
- **Parameters:**
  - `policy_path` (optional string): Path to custom policy JSON file.
  - `spec` (optional object): Session specification object to evaluate.
  - `store_path` (optional string): Path to custom session store JSON file.
  - `grant_id` (optional string): PEP authorization grant.

### 3. Invariants & Known Constraints
- **SSP1 (Root & Greeter Boundaries)**: UID 0 root sessions are blocked by default unless the username is in `allowed_root_users`. Greeter class sessions must run as system users (< 1000) or authorized greeters (`lightdm`, `gdm`, etc.).
- **SSP2 (Session Type & Class Gating)**: Agent class sessions must have `session_type: "ai_agent"`. Greeter sessions must declare a valid display and cannot be remote.
- **SSP3 (Seat & Display Hardware)**: Remote sessions (`remote_host.is_some()`) are forbidden from attaching to console `seat0` unless `allow_remote_seat0` is enabled. VT numbers must be in range $[1 \dots 64]$.
- **SSP4 (Environment Sanitization)**: Variables `LD_PRELOAD`, `LD_LIBRARY_PATH`, `IFS`, `NODE_OPTIONS`, `PYTHONPATH`, `RUBYOPT`, `PERL5OPT` are strictly rejected.
- **SSP5 (Capacity Quotas)**: Enforces maximum active sessions per user ($\le 32$) and maximum total sessions ($\le 1,024$).
- **SSP6 (Agent Sandboxing)**: AI Agent sessions are forbidden from running with privileged UID 0.
- **SSP7 (Policy Modes & Limits)**: Enforcing (blocks on fatal violations), Audit (logs violations but allows), Permissive (telemetry only). Policy files capped at $\le 64\text{ KiB}$.
