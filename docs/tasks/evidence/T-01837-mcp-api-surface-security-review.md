# T-01837: Network Bootstrap / MCP/API Surface: Security Review

## 1. Overview
- **Task ID**: `T-01837`
- **Sub-Epic**: 4 (MCP/API Surface)
- **Goal**: Perform comprehensive security review and threat analysis of the MCP tools (`aios.network.*`).

---

## 2. Threat Modeling & Abuse Scenarios

### Scenario 1: Path Traversal via Optional Path Overrides (`sysfs_path`, `procfs_path`, `resolv_path`)
- **Attack Vector**: An AI agent or remote MCP caller supplies crafted paths such as `../../etc/shadow` or strings containing null bytes (`\0`) or newlines (`\n`) to read sensitive files or cause path confusion.
- **Analysis**: If accepted unscrubbed, custom root paths could allow unauthorized directory inspection.
- **Mitigation in Code**:
  - `resolve_network_service` explicitly verifies:
    - Path length $\le 1024$ characters.
    - No control characters (`c.is_control()`).
  - Underlying service utilizes `read_bounded_string` with strict size bounds (64 KB for sysfs/resolv, 1 MB for route table).
- **Verdict**: Mitigated.

### Scenario 2: Interface Name Injection (`aios.network.show`, `aios.network.up`, `aios.network.down`)
- **Attack Vector**: An untrusted caller provides an interface name with shell metacharacters (e.g. `eth0; rm -rf /`), directory traversal characters (`../../dev/null`), or excessive length (> 15 chars).
- **Analysis**: Passing malicious interface names to filesystem operations or system hooks could lead to arbitrary path access or execution.
- **Mitigation in Code**:
  - `validate_interface_name` is invoked before any service lookup or link mutation.
  - Constrains names to non-empty, $\le 15$ characters matching `^[a-zA-Z0-9_.-]+$`.
- **Verdict**: Mitigated.

### Scenario 3: Unauthorized Link State Mutation (`aios.network.up`, `aios.network.down`)
- **Attack Vector**: An unprivileged agent triggers interface link deactivation (`down`) or activation (`up`) without authorization or audit logging.
- **Analysis**: Disrupting host network connectivity can sever agent communications and impact system availability.
- **Mitigation in Code**:
  - Both `aios.network.up` and `aios.network.down` are routed through `dispatch::recorded_call`.
  - Target interface is explicitly supplied as `target` parameter in the audit event.
  - The audit ring commits actor identity, timestamp, tool name, arguments, and outcome.
- **Verdict**: Mitigated.

### Scenario 4: Audit Evasion & Silent Failures
- **Attack Vector**: An attacker attempts to exploit error paths to trigger unrecorded execution or unhandled exceptions that crash the MCP server.
- **Analysis**: MCP stdio crashes disconnect the agent session.
- **Mitigation in Code**:
  - All errors (including invalid arguments and missing interfaces) are captured within the closure and returned via `dispatch::recorded_call`, writing an honest failure audit row per ADR-0035 §F-2.
- **Verdict**: Mitigated.

---

## 3. Review Conclusion
Zero open vulnerabilities or policy bypasses. The MCP/API surface strictly adheres to invariants `NMCP1` through `NMCP6`.
