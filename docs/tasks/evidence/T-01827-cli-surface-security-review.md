# T-01827: Network Bootstrap / CLI Surface: Security Review

## 1. Overview
- **Task ID**: `T-01827`
- **Sub-Epic**: 3 (CLI Surface)
- **Goal**: Perform comprehensive security review and threat analysis of the CLI surface (`aiosh net` / `aiosh network`).

---

## 2. Threat Modeling & Abuse Scenarios

### Scenario 1: Path Injection via Custom Root Flags (`--sysfs`, `--procfs`, `--resolv`)
- **Attack Vector**: An attacker supplies deeply nested paths, control characters (e.g., null bytes `\0`, newlines `\n`), or excessively long strings (> 1024 chars) to `--sysfs`, `--procfs`, or `--resolv` to trigger buffer overflow, path confusion, or denial-of-service.
- **Analysis**: Unbounded paths could cause excessive memory allocation or unexpected system calls.
- **Mitigation in Code**:
  - `cmd_network` validates that `p.len() <= 1024` for all path flags.
  - `p.chars().any(|c| c.is_control())` rejects control characters.
  - Violations return exit code 2 and emit an audit row with error code `PATH_TOO_LONG` or `PATH_CONTAINS_CONTROL_CHAR`.
- **Verdict**: Mitigated.

### Scenario 2: Interface Name Injection in `show`, `up`, `down`
- **Attack Vector**: An attacker passes malicious strings as interface names, such as `eth0; rm -rf /`, `../../sys/class/net/eth0`, or names with control characters.
- **Analysis**: If passed unchecked to filesystem lookups or system commands, this could lead to path traversal or command injection.
- **Mitigation in Code**:
  - `validate_interface_name` enforces:
    - Non-empty name.
    - Maximum 15 characters (Linux `IFNAMSIZ - 1`).
    - Character set strictly matching `^[a-zA-Z0-9_.-]+$`.
    - No `/`, `\`, `\0`, or path traversal components (`.` or `..` as standalone names).
  - Rejection produces exit code 2 with error code `INVALID_INTERFACE_NAME` and emits an audit event.
- **Verdict**: Mitigated.

### Scenario 3: ANSI Terminal Escape Sequence Injection
- **Attack Vector**: Interface names, routes, or error strings from untrusted sources contain VT100/ANSI escape sequences (e.g., `\x1b[2J\x1b[H` or title-setting sequences) to alter operator terminal state or disguise actions.
- **Analysis**: Raw output to stdout/stderr in interactive shells can deceive operators.
- **Mitigation in Code**:
  - All outputs in human mode pass through `sanitize_terminal`, stripping or escaping non-printable and ANSI escape sequences.
- **Verdict**: Mitigated.

### Scenario 4: State Mutation Without Audit Trail
- **Attack Vector**: An operator or agent triggers `aiosh net up <iface>` or `aiosh net down <iface>` without logging.
- **Analysis**: State mutations on network interfaces must be traceable in the central audit ledger.
- **Mitigation in Code**:
  - `classify_and_emit(&mut ctx, "network", "up", ...)` and `classify_and_emit(&mut ctx, "network", "down", ...)` are executed for every call.
  - Both success and failure outcomes are recorded with interface name and error details.
- **Verdict**: Mitigated.

### Scenario 5: Silent Failures in Automated Scripts
- **Attack Vector**: Automation parses CLI output and assumes success if stdout is empty or errors are unformatted.
- **Mitigation in Code**:
  - `--json` provides a deterministic envelope: `{"code": <int>, "data": ..., "error": ...}`.
  - Exit codes strictly adhere to POSIX conventions: 0 for success, 1 for operational failures (e.g. interface not found), 2 for argument/validation errors.
- **Verdict**: Mitigated.

---

## 3. Review Conclusion
Zero open policy bypasses or vulnerabilities identified. The CLI surface conforms to AIOS security architecture invariants `NCLI1` through `NCLI6`.
