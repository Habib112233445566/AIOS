# Specification: Secrets Handling Security Policy (SPEC-SECRETS-POLICY)

- **Status**: IMPLEMENTED & HARDENED
- **Date**: 2026-10-03
- **Subsystem**: Phase 2 Security Kernel & PEP Fabric / Secrets Handling Security Policy
- **Binding**: ADR-0035 §F-2

## 1. Overview
The Secrets Handling security policy subsystem establishes declarative, rule-based governance over secret registration, retrieval, and payload rotation across the AIOS vault fabric.

## 2. Policy Structure

```json
{
  "version": "1.0.0",
  "mode": "enforcing",
  "disallow_global_secrets": false,
  "max_payload_bytes": 65536,
  "prohibited_kinds": [],
  "require_expose_flag": true,
  "max_lifetime_seconds": 2592000
}
```

### 2.1 Rust Data Models
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretPolicyMode {
    Enforcing,
    Permissive,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum SecretPolicyVerdict {
    Permit,
    PermitWithWarning { warning: String },
    Deny { reason: String, code: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretSecurityPolicy {
    pub version: String,
    pub mode: SecretPolicyMode,
    pub disallow_global_secrets: bool,
    pub max_payload_bytes: usize,
    pub prohibited_kinds: Vec<SecretKind>,
    pub require_expose_flag: bool,
    pub max_lifetime_seconds: u64,
}
```

## 3. Enforcement Modes
- **Enforcing**: Rule violations return `SecretPolicyVerdict::Deny` fail-closed with explicit error codes (`SECPOL_ERR_*`).
- **Permissive**: Rule violations return `SecretPolicyVerdict::PermitWithWarning` and log warning telemetry to `AuditRing`.
- **Disabled**: Policy evaluation returns `SecretPolicyVerdict::Permit` unconditionally.

## 4. Policy Invariants & Error Constants
- `SECPOL_ERR_GLOBAL_DISALLOWED`: Global scope disallowed when `disallow_global_secrets` is active.
- `SECPOL_ERR_KIND_PROHIBITED`: Secret registration rejected if `kind` is in `prohibited_kinds`.
- `SECPOL_ERR_PAYLOAD_TOO_LARGE`: Secret payload exceeds policy `max_payload_bytes`.
- `SECPOL_ERR_EXPOSE_REQUIRED`: Raw secret access denied when `require_expose_flag` is violated.
- `SECPOL_ERR_DENIED`: Caller scope lacks authorization for the target secret scope.
- `SECPOL_ERR_VALIDATION`: Invalid version, traversal path, control character, or out-of-bounds parameters.
- `SECPOL_ERR_IO`: File read/stat/write failures.
- `SECPOL_ERR_PARSE`: JSON serialization or deserialization syntax failures.

## 5. Environment Variable Overrides
The runtime policy dynamically honors the following environment variables on startup:
- `AIOS_SECRETS_POLICY_PATH`: Custom path to policy JSON file (traversal checked).
- `AIOS_SECRETS_POLICY_MODE`: Mode override (`enforcing`, `permissive`, `disabled`).
- `AIOS_SECRETS_POLICY_DISALLOW_GLOBAL`: Boolean flag (`1`, `true`, `yes` / `0`, `false`, `no`).
- `AIOS_SECRETS_POLICY_MAX_PAYLOAD`: Clamped payload limit in bytes (`1..=1048576`).
- `AIOS_SECRETS_POLICY_REQUIRE_EXPOSE`: Boolean flag (`1`, `true`, `yes` / `0`, `false`, `no`).

## 6. Operator Interfaces & Examples

### 6.1 CLI Commands

#### Show Active Policy
```bash
aiosh secret policy show
aiosh secret policy show --json
```

#### Validate Policy File
```bash
aiosh secret policy check --policy config/secrets_policy.json
```

#### Update Policy Mode
```bash
aiosh secret policy set-mode --mode permissive
aiosh secret policy set-mode --mode enforcing --policy config/secrets_policy.json
```

### 6.2 MCP Tool Interface (`aios.secret.policy`)

#### Inspect Policy via MCP
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.secret.policy",
    "arguments": {
      "action": "show"
    }
  }
}
```

#### Check Policy Integrity via MCP
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "aios.secret.policy",
    "arguments": {
      "action": "check",
      "policy_path": "secrets_policy.json"
    }
  }
}
```

#### Set Policy Mode via MCP
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "aios.secret.policy",
    "arguments": {
      "action": "set-mode",
      "mode": "permissive"
    }
  }
}
```

## 7. Security Constraints & Known Limitations
1. **File Size Bound**: Policy file size is strictly capped at 64 KiB (`MAX_SECRET_SECURITY_POLICY_BYTES = 65536`) to protect memory.
2. **Path Sanitization**: Directory traversal (`..`), null bytes (`\0`), and control characters in file paths are rejected fail-closed.
3. **Prohibited Kinds Ceiling**: Maximum 64 prohibited secret kinds allowed to ensure predictable constant-time evaluation.
4. **Atomic Persistence**: Policy writes write to an adjacent temporary file before atomic renaming to prevent partial corruption.
5. **Permissive Auditing**: When operating in `permissive` mode, violations do not block execution but write an auditable warning event to the append-only audit ring.

## 8. Sub-Epic Evidence Trail
- [T-02661 Research](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02661-security-policy-research.md)
- [T-02662 Specification](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02662-security-policy-specification.md)
- [T-02663 Scaffold](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02663-security-policy-scaffold.md)
- [T-02664 Implementation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02664-security-policy-implementation.md)
- [T-02665 Unit Test](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02665-security-policy-unit-test.md)
- [T-02666 Integration](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02666-security-policy-integration.md)
- [T-02667 Security Review](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02667-security-policy-security-review.md)
- [T-02668 Hardening](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02668-security-policy-hardening.md)
- [T-02669 Documentation](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02669-security-policy-documentation.md)
- [T-02670 Verification & Evidence](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02670-security-policy-verification-evidenc.md)
