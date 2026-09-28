# T-02467: Sandbox Enforcement Security Policy Security Review

## 1. Security Review Scope
This review evaluates the security policy implementation (`sandbox_policy.rs`, `sandbox_service.rs`, CLI and MCP surfaces) for resistance against bypasses, injection attacks, and resource abuse.

---

## 2. Abuse Scenarios & Mitigations

### Abuse Scenario 1: Command Path & Extension Evasion
- **Threat**: An attacker attempts to bypass `prohibited_commands` (e.g. `rm`) by invoking `/bin/rm`, `rm.exe`, `RM`, or `C:\tools\rm.exe`.
- **Mitigation**:
  - `SandboxSecurityPolicy::evaluate` trims, lowercases, extracts the base filename via `Path::file_name`, and strips `.exe` before checking against `prohibited_commands`.
  - Verified: `/bin/rm`, `RM.EXE`, and `rm` are all correctly recognized and denied.

### Abuse Scenario 2: Environment Variable Case Variation
- **Threat**: An attacker attempts to smuggle `ld_preload` or `Ld_Preload` past ASCII case-sensitive filters.
- **Mitigation**:
  - `SandboxSecurityPolicy::evaluate` employs `eq_ignore_ascii_case` against all injected environment variables.
  - Case variations of `LD_PRELOAD`, `DYLD_INSERT_LIBRARIES`, `PYTHONPATH`, and `NODE_OPTIONS` are strictly denied.

### Abuse Scenario 3: Bounded Policy Storage & DoS Prevention
- **Threat**: Supplying a multi-gigabyte crafted JSON file as `--path` to exhaust heap memory.
- **Mitigation**:
  - `load_from_path` checks file metadata size before reading; files exceeding `MAX_SANDBOX_SECURITY_POLICY_BYTES` (64 KiB) are rejected immediately with `SANDBOXPOL_ERR_VALIDATION`.

### Abuse Scenario 4: Fail-Closed Audit Trail Non-Repudiation
- **Threat**: Denied executions fail silently without leaving an audit record.
- **Mitigation**:
  - When `SandboxService::execute` denies an execution request due to policy violation, it writes an audit row into `AuditRing` with `outcome: "denied"`, the specific policy denial reason, and `policy_revision`.

---

## 3. Verdict
All threat vectors are adequately defended. Status: **PASS / SECURE**.
