# T-00687 — Repository Health / observability: Security Review

## Security Audit & Threat Modeling
- **Component**: `Repository Health / observability`
- **Scope**: Observability diagnostics, health reporting, CLI/MCP surfaces, and data collection handlers.

### Abuse Scenarios & Mitigations
1. **Command / Argument Injection**:
   - *Threat*: Untrusted parameters injected into underlying `git status` commands.
   - *Mitigation*: Subprocess execution uses `std::process::Command` with fixed positional arguments (`["status", "--porcelain=v2"]`), bypassing shell expansion completely.
2. **Denial of Service via Giant File Systems / Buffer Bloat**:
   - *Threat*: Pathological directory nesting or massive untracked files exhausting memory during health diagnostics.
   - *Mitigation*: Directory scanning bounds file size checking with 16 MiB default limits, ignores `.git`/`target`/`node_modules`, and clamps reported detail vectors (`.take(50)`).
3. **Information Disclosure**:
   - *Threat*: Emitting sensitive uncommitted secrets in health reports across MCP tools.
   - *Mitigation*: Health checks only report file names/paths and aggregate metrics, never dumping raw file content into reports.
4. **State Gating & Mutations**:
   - *Threat*: Unauthorized mutation of repo state via health endpoints.
   - *Mitigation*: All `repo health` checks and metrics are strictly read-only and non-mutating.

### Policy Verdict
- **Status**: PASS. No unmitigated policy bypasses or security regressions identified.
