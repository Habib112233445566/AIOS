# T-02417: Sandbox Enforcement Core Service Security Review

## 1. Scope & Objective
Security review of the **Sandbox Enforcement Core Service** (`SandboxService`) in `aiosh-core/src/sandbox_service.rs` and its exposure in `aiosh-mcp/src/main.rs`.

## 2. Threat Modeling & Abuse Scenarios

### Abuse Scenario A: PEP Enforcement Evasion
- **Threat**: An unauthorized agent attempts to execute an uncontained or sandboxed command without presenting a cryptographic PEP grant token.
- **Analysis & Mitigation**: When `config.enforce_pep_grants` is active, `SandboxService::execute` strictly verifies the presence of `request.pep_grant_id`. Calls lacking a grant fail closed with `ERR_SANDBOX_PEP_UNAUTHORIZED`.
- **Verdict**: Mitigated.

### Abuse Scenario B: Default Profile Tampering / Deletion
- **Threat**: An adversary deletes `standard` or `strict` profile to induce a missing profile state or fallback to loose containment.
- **Analysis & Mitigation**: `SandboxService::remove_profile` validates the target name against protected factory profiles (`standard`, `strict`, `permissive`) and aborts with `ERR_SANDBOX_CANNOT_DELETE_DEFAULT`.
- **Verdict**: Mitigated.

### Abuse Scenario C: Output Flooding Denial of Service
- **Threat**: A sandboxed process generates infinite or massive volumes of stdout/stderr data to exhaust service memory.
- **Analysis & Mitigation**: Output streams are intercepted and bounded at `config.max_output_capture_bytes` (10 MiB default), preventing buffer bloat and host daemon crash.
- **Verdict**: Mitigated.

### Abuse Scenario D: Audit Evasion via Non-Zero Exit
- **Threat**: An adversary deliberately causes process termination hoping to bypass audit recording.
- **Analysis & Mitigation**: Audit records are written unconditionally when `audit_enabled` is true, capturing both successful and non-zero exit outcomes with duration and error telemetry.
- **Verdict**: Mitigated.

## 3. Conclusion
No vulnerabilities or policy bypasses exist in the core service implementation.
