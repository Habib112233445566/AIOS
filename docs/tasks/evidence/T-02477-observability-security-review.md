# Task T-02477 Evidence: Sandbox Observability Security Review

## Goal
Conduct a thorough security review of the Sandbox Enforcement Observability subsystem (`SandboxObservabilityReport`, CLI `aiosh sandbox stats`, and MCP `aios.sandbox.stats`).

## Security Analysis & Review Matrix

| Vector / Component | Risk Identified | Mitigation Applied | Residual Risk |
|---|---|---|---|
| Untrusted Audit Strings | Injection of terminal escape sequences via recorded profile/outcome text | All strings are scrubbed with `sanitize_telemetry_text`, replacing control chars (< 0x20, 0x7F) with `?` and truncating to `MAX_TELEMETRY_TEXT_LEN` (256 bytes) | None (terminal safe) |
| Memory Bloat / DoS | Cardinality explosion in outcome/profile frequency maps | Bounded collection enforced via `MAX_OUTCOME_DISTRIBUTION_ENTRIES` (128 max entries) | Negligible |
| Unauthorized Mutation | Tampering with sandbox security policy via observability surface | Observability surfaces are strictly read-only; no mutating methods exist on `SandboxObservabilityReport` | None |
| Side-Channel Leakage | Unintended disclosure of secret environment variables or command arguments | Observability reports only aggregate outcome tallies, profile names, and platform capabilities; raw command arguments and environment vars are never exposed | None |
| Storage & Concurrency | Deadlocks or file locks when reading audit log | `AuditRing::open(OpenOptions::default()).ok()` gracefully degrades to an empty record stream if the ring is locked or uninitialized | None |

## Abuse Scenarios Evaluated

### Scenario 1: Malicious ANSI Escape Sequence Injection
- **Attack Vector**: An adversary passes crafted profile names or command names containing ANSI sequences (e.g. cursor movement, terminal title reset) designed to spoof output in administrative logs.
- **Verification**: `sanitize_telemetry_text` strips/sanitizes all control characters below 0x20, preventing terminal manipulation.

### Scenario 2: Cardinality Exhaustion Attack
- **Attack Vector**: An attacker attempts to flood the audit log with millions of unique synthetic outcome identifiers to consume all memory on the host during stats generation.
- **Verification**: `record_outcome` and `record_profile_execution` check `len() >= MAX_OUTCOME_DISTRIBUTION_ENTRIES` and reject new keys beyond 128 entries.

### Scenario 3: Privilege Escalation via Host Capability Inspection
- **Attack Vector**: Attacker attempts to forge kernel capability reporting (`seccomp`, `landlock`).
- **Verification**: `HostCapabilities::detect()` reads directly from the compiled OS platform targets and kernel interfaces, returning immutable booleans that cannot be overridden by callers.

## Policy Bypass Evaluation
No policy bypasses or security vulnerabilities were identified in the observability implementation.
