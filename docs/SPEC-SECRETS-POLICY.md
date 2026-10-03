# Specification: Secrets Handling Security Policy (SPEC-SECRETS-POLICY)

- **Status**: APPROVED
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
#[serde(tag = "type", rename_all = "snake_case")]
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
- `SECPOL_ERR_EXPIRED_LIFETIME`: Secret exceeds configured maximum lifetime.
- `SECPOL_ERR_VALIDATION`: Invalid version, traversal path, or out-of-bounds parameters.

## 5. Operator Interfaces

### 5.1 CLI Commands
```bash
aiosh secret policy [show|check|set-mode] [--policy <path>] [--mode <enforcing|permissive|disabled>] [--json]
```

### 5.2 MCP Tool Interface
```json
{
  "name": "aios.secret.policy",
  "description": "Inspect, validate, or evaluate Secrets Handling security policy",
  "inputSchema": {
    "type": "object",
    "properties": {
      "action": {"type": "string", "enum": ["show", "check", "set_mode", "evaluate"]},
      "policy_path": {"type": "string"},
      "mode": {"type": "string", "enum": ["enforcing", "permissive", "disabled"]}
    },
    "additionalProperties": false
  }
}
```
