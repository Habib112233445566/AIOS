# Privilege Escalation Prevention Security Policy Guide (SPEC-PRIVILEGE-POLICY)

- **Status**: ACTIVE
- **Subsystem**: `aiosh-core`, `aiosh-cli`, `aiosh-mcp`
- **Updated**: 2026-10-02

## 1. Overview
The Privilege Escalation Prevention security policy subsystem provides declarative governance over all privilege elevation attempts across user sessions, agents, and system processes.

## 2. Policy Structure
```json
{
  "version": "1.0.0",
  "mode": "enforcing",
  "disallowed_elevation_targets": [
    "system_kernel"
  ],
  "prohibited_capabilities": [
    "kernel_module_load"
  ],
  "require_grant_token": true,
  "max_grant_duration_seconds": 3600,
  "actor_tier_ceilings": {
    "contractor": "user",
    "ci_runner": "operator"
  }
}
```

## 3. Enforcement Modes
- **Enforcing**: Invariant violations fail-closed returning `PRIVESCPOL_ERR_DENIED`.
- **Permissive**: Invariant violations generate a `PermitWithWarning` verdict and emit classified warnings to `AuditRing`.
- **Disabled**: Policy evaluation is bypassed.

## 4. Operator Commands
### CLI Inspection
```bash
aiosh privilege policy [--mode <enforcing|permissive|disabled>] [--policy-file <PATH>] [--json]
```

### MCP Tool Invocation
Tool name: `aios.privilege.policy`
Parameters:
- `policy_path` (string, optional)
- `mode` (string: "enforcing", "permissive", "disabled", optional)

Example request:
```json
{
  "name": "aios.privilege.policy",
  "arguments": {
    "mode": "enforcing"
  }
}
```
