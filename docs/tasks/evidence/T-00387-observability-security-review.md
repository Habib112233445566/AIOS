# T-00387 — Dependency & Toolchain Pinning / observability: Security Review

## 1. Review Scope
This security review analyzes the observability pipelines, runtime telemetry capture, and error diagnostic aggregation for Dependency & Toolchain Pinning in AIOS.

## 2. Threat Analysis & Security Controls
1. **Subprocess Injection Defense**:
   - Diagnostic probes invoke fixed executables (`rustc`, `python3`, `python`, `node`) with static argument vectors (`["-V"]`, `["-v"]`).
   - Direct execution via `std::process::Command` without shell wrapper invocation prevents command injection attacks.
2. **Untrusted Binary Output Sanitization**:
   - Output bytes from external processes are decoded using `String::from_utf8_lossy`, preventing crashes on malformed UTF-8 sequences.
   - All captured diagnostic strings are escaped by `serde_json` when writing to the SQLite WAL audit ring, preventing log injection or record splitting attacks.
3. **Execution Bounds & Timeouts**:
   - External command execution is bounded by 15-second timeouts with explicit child process termination and reap to prevent hanging processes or resource leaks.

## 3. Abuse Scenarios & Mitigations

### Abuse Scenario 1: Audit Log Injection via Crafty Binary Output
- **Attack Vector**: A custom toolchain binary emits fake JSON lines containing forged audit headers.
- **Mitigation**: The audit engine writes strongly-typed JSON records where process output is strictly encapsulated inside string literals (`outcome_detail`). The cryptographic SHA-256 hash chain guarantees record integrity.

### Abuse Scenario 2: Denial of Service via Slow/Hanging Probes
- **Attack Vector**: A malicious `rustc` binary blocks indefinitely on stdin/stdout.
- **Mitigation**: The execution runner uses a 15-second timeout, kills the process with `child.kill()`, and reaps the child process to prevent zombie processes.

## 4. Conclusion
No known security bypass remains open. Observability telemetry is securely captured, escaped, and persisted without introducing injection vulnerabilities.
