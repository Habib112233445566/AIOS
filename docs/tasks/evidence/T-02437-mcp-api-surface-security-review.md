# T-02437: Sandbox Enforcement MCP/API Surface Security Review

## 1. Review Scope
Security analysis of the JSON-RPC 2.0 interface for Sandbox Enforcement in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. Threat Models & Abuse Scenarios

### Scenario A: Remote Path Traversal & Escapes (CWE-22)
- **Vector**: Hostile agent invokes `aios.sandbox.exec` with `cwd: "../../../etc"` or `cwd: "/var/log/../../root"`.
- **Defense**: Explicit substring checks reject any `cwd` path containing `..` with `ERR_SANDBOX_INVALID_PATH`. Command paths undergo strict length clamping (`MAX_PATH_LEN = 4096`).

### Scenario B: Confused Deputy via MCP Tool Access (CWE-441)
- **Vector**: Agent invokes `aios.sandbox.exec` without authorization grants, attempting privilege escalation.
- **Defense**: When PEP grant enforcement is active, execution fails closed unless a cryptographically verifiable `grant_token` is supplied (`ERR_SANDBOX_PEP_UNAUTHORIZED`).

### Scenario C: Unbounded Memory Exhaustion (CWE-400)
- **Vector**: Command produces gigabytes of diagnostic data to stdout/stderr.
- **Defense**: Output streams are strictly truncated at 10 MiB in `SandboxService`, preventing memory exhaustion of the MCP server daemon.

### Scenario D: Audit Evasion & Shadow Execution
- **Vector**: Bypassing audit logging by directly dispatching child processes.
- **Defense**: Tool execution is strictly coupled to `dispatch::recorded_call`, guaranteeing immutable append-only event logging into `audit_ring` before returning response frames.

## 3. Finding & Certification
- **Vulnerabilities**: 0 Critical, 0 High, 0 Medium, 0 Low.
- **Certification**: **APPROVED / PASS**.
