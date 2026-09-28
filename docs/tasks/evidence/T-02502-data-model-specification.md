# Task T-02502 Evidence: Privilege Escalation Prevention Data Model Specification

## Goal
Specify the formal data contracts, type schemas, invariant validation rules, and evaluation verdicts for Privilege Escalation Prevention.

## 1. Domain Types & Schemas

### `PrivilegeLevel`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeLevel {
    Guest = 0,
    User = 10,
    Operator = 20,
    Admin = 30,
    SystemKernel = 40,
}
```

### `PrivilegeCapability`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegeCapability {
    ProcessSpawn,
    NetworkConnect,
    NetworkListen,
    FilesystemWrite,
    MemoryInspect,
    AuditLogAdmin,
    SystemReboot,
    KernelModuleLoad,
}
```

### `PrivilegeContext`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeContext {
    pub actor_id: String,
    pub active_level: PrivilegeLevel,
    pub capabilities: HashSet<PrivilegeCapability>,
    pub elevation_grant_id: Option<String>,
    pub is_elevation_active: bool,
    pub session_id: Option<String>,
}
```

### `PrivilegeTransitionRequest`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivilegeTransitionRequest {
    pub actor_id: String,
    pub from_level: PrivilegeLevel,
    pub target_level: PrivilegeLevel,
    pub requested_capabilities: Vec<PrivilegeCapability>,
    pub grant_id: Option<String>,
}
```

### `PrivilegeEscalationVerdict`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivilegeEscalationVerdict {
    Allowed,
    GrantRequired { reason: String },
    Denied { reason: String },
}
```

## 2. Invariants & Rulesets
1. **PRIVESC1 (Monotonic Escalation Law)**: Moving from a lower level to a higher level requires an explicit capability grant. Without a grant, evaluation yields `GrantRequired`.
2. **PRIVESC2 (Kernel Tier Immutability)**: Userspace transitions into `PrivilegeLevel::SystemKernel` are unconditionally `Denied`.
3. **PRIVESC3 (Capability Ceilings)**: High-risk capabilities (`KernelModuleLoad`, `SystemReboot`, `AuditLogAdmin`) require minimum `Admin` tier.
4. **PRIVESC4 (Context Sanitization)**: `actor_id` cannot be empty, exceed 128 characters, or contain ASCII control characters.
5. **PRIVESC5 (Capability Set Limits)**: No context may hold more than `MAX_CAPABILITIES_COUNT` (32) distinct capabilities.

## 3. Standard Error Codes
- `PRIVESC_ERR_INVALID_ACTOR`: Actor identifier is empty or invalid.
- `PRIVESC_ERR_KERNEL_TIER_IMMUTABLE`: Transition to SystemKernel requested from userspace.
- `PRIVESC_ERR_CAPABILITY_OVERFLOW`: Capability count exceeds maximum allowable ceiling.
- `PRIVESC_ERR_CAPABILITY_UNAUTHORIZED`: Capability requested exceeds active tier ceiling without grant.
- `PRIVESC_ERR_UNAUTHORIZED_ELEVATION`: Escalation attempted without a valid PEP grant.
