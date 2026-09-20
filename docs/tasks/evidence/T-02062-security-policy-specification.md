# Specification: Capability Security Policy (T-02062)

## 1. Scope & Purpose
The **Capability Security Policy** subsystem defines the formal security rules, boundary checks, and enforcement logic governing capability issuance and attenuation within the AIOS Security Kernel (`CAPSEC1..CAPSEC6`).

---

## 2. Architecture & Data Structures

### 2.1 Enums & Modes
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityPolicyMode {
    Enforcing,
    Audit,
    Permissive,
}
```

### 2.2 Policy Configuration Struct
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilitySecurityPolicy {
    pub mode: CapabilityPolicyMode,
    pub max_attenuation_depth: usize,
    pub disallowed_rights_by_subject_prefix: HashMap<String, Vec<CapabilityRight>>,
    pub prohibited_path_prefixes: Vec<String>,
    pub prohibited_network_hosts: Vec<String>,
    pub prohibited_tools: Vec<String>,
    pub require_temporal_bounds: bool,
    pub max_validity_duration_seconds: Option<i64>,
    pub max_invocations_ceiling: Option<u64>,
    pub max_bytes_ceiling: Option<u64>,
}
```

### 2.3 Policy Verdict & Violations
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityPolicyViolation {
    pub rule_id: String,
    pub description: String,
    pub fatal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityPolicyVerdict {
    pub allowed: bool,
    pub mode: CapabilityPolicyMode,
    pub violations: Vec<CapabilityPolicyViolation>,
    pub evaluated_at: String,
}
```

---

## 3. Policy Evaluation Invariants & Rules

| Rule ID | Name | Description | Failure Behavior |
| :--- | :--- | :--- | :--- |
| `CAPSEC-01` | Attenuation Depth | `depth <= max_attenuation_depth` (default: 8, max: 64). | `CapabilityPolicyViolation` with `rule_id = "CAPSEC_DEPTH_EXCEEDED"`. |
| `CAPSEC-02` | Prohibited Paths | Filesystem scope path must not start with any `prohibited_path_prefixes` (e.g. `/etc`, `/proc`, `/sys`, `/dev`, `C:\Windows\System32`). | `CapabilityPolicyViolation` with `rule_id = "CAPSEC_PROHIBITED_PATH"`. |
| `CAPSEC-03` | Prohibited Network | Network scope host must not match `prohibited_network_hosts` (e.g. `169.254.169.254`, `metadata.google.internal`). | `CapabilityPolicyViolation` with `rule_id = "CAPSEC_PROHIBITED_HOST"`. |
| `CAPSEC-04` | Prohibited Tools | Tool scope `tool_name` must not be in `prohibited_tools` for non-admin subjects. | `CapabilityPolicyViolation` with `rule_id = "CAPSEC_PROHIBITED_TOOL"`. |
| `CAPSEC-05` | Disallowed Rights | If subject prefix matches a rule in `disallowed_rights_by_subject_prefix`, no requested right may be in that disallowed set. | `CapabilityPolicyViolation` with `rule_id = "CAPSEC_DISALLOWED_RIGHT"`. |
| `CAPSEC-06` | Temporal Ceiling | If `require_temporal_bounds` is true, `expires_at` must be present and duration must not exceed `max_validity_duration_seconds`. | `CapabilityPolicyViolation` with `rule_id = "CAPSEC_INVALID_TEMPORAL_BOUND"`. |
| `CAPSEC-07` | Quota Ceilings | If ceilings exist, `max_invocations` and `quota_bytes` must not exceed `max_invocations_ceiling` and `max_bytes_ceiling`. | `CapabilityPolicyViolation` with `rule_id = "CAPSEC_QUOTA_CEILING_EXCEEDED"`. |

---

## 4. Operational Methods

### 4.1 `validate(&self) -> Result<(), String>`
Validates the internal consistency of the policy:
- `1 <= max_attenuation_depth <= 64`.
- `prohibited_path_prefixes` entries must be non-empty and well-formed.
- `max_validity_duration_seconds` (if set) must be $> 0$.

### 4.2 `evaluate_issuance(...) -> CapabilityPolicyVerdict`
Evaluates a proposed root capability against the security policy before insertion into the registry.

### 4.3 `evaluate_attenuation(...) -> CapabilityPolicyVerdict`
Evaluates a proposed child capability derivation against the security policy and derivation depth limit.

---

## 5. Audit Effects & Error Handling
- In `CapabilityPolicyMode::Enforcing`: If any fatal violation occurs, `allowed` is false, and the operation returns `Err(CapabilityError::Validation(...))` or `Err(CapabilityError::Scope(...))` containing the rule ID and reason.
- In `CapabilityPolicyMode::Audit`: Violations are recorded in the verdict, but `allowed` evaluates to `true` (non-blocking, logged for telemetry).
- In `CapabilityPolicyMode::Permissive`: All evaluations succeed with empty violations.
