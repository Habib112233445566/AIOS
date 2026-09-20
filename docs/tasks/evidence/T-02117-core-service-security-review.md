# T-02117: Security Review — PEP Decision Engine Core Service

## Scope & Objective
Security review of `PepDecisionService` and `PepDecision` data model to verify input validation, resistance to abuse, fail-closed invariants, and audit integrity.

## Threat Analysis & Abuse Scenarios

### 1. Path Traversal Evasion (`fs:/allowed/../../etc/shadow`)
- **Threat**: An agent or user submits a path traversal sequence inside a resource URI to evade prefix or exact-match security rules.
- **Verification**: `PepRequest::new` inspects the resource string for `..` components and control characters (`\0`, `\n`, `\r`). Any traversal attempt is rejected immediately with `PepDecisionError::InvalidResource`.
- **Status**: MITIGATED.

### 2. Denial of Service via Policy Flooding (Memory Exhaustion)
- **Threat**: A compromised service or malicious script attempts to register millions of policy rules to induce an Out-Of-Memory (OOM) crash in the kernel.
- **Verification**: `MAX_RULES_IN_SERVICE = 5000` is enforced in `PepDecisionService::add_rule`. Attempts to exceed this capacity return `PepDecisionError::CapacityExceeded`.
- **Status**: MITIGATED.

### 3. Policy Store Poisoning & Corrupt File Handling
- **Threat**: Tampering with or corrupting `pep_policies.json` causes the service to crash or overwrite existing policies destructively.
- **Verification**: `load_or_recover` uses non-destructive quarantine: the corrupted file is renamed to `<path>.bak.<timestamp>` with restricted mode `0600` on Unix before a fresh store is initialized.
- **Status**: MITIGATED.

### 4. Policy Bypass via Ambiguous Rule Evaluation
- **Threat**: Unmatched requests or conflicting rules result in permissive access.
- **Verification**: Invariant `PEPDEC1` mandates fail-closed default deny. If no rule matches, the outcome is strictly `Deny`. In `DenyOverrides`, any matching `Deny` rule overrides any number of `Permit` rules.
- **Status**: MITIGATED.

### 5. Audit Row Suppression
- **Threat**: Evaluation execution fails to record decisions in the audit trail.
- **Verification**: All MCP evaluations are wrapped in `dispatch::recorded_call`, ensuring every invocation writes an immutable record to the SQLite audit ring.
- **Status**: MITIGATED.

## Finding Summary
- Zero open vulnerabilities.
- Fail-closed invariant strictly preserved across all evaluation paths.
