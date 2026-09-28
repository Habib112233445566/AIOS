# Specification: Privilege Escalation Prevention Core Service

## 1. Overview
The Privilege Escalation Prevention Core Service (`PrivilegeService`) provides runtime management, evaluation, and enforcement of privilege contexts. It ensures that actors execute within their authorized tiers, that dynamic elevation requires valid PEP grant tokens, and that sensitive capabilities are guarded against unauthorized invocation.

---

## 2. Invariants & Rulesets

| Invariant | Name | Description |
|---|---|---|
| `PRIVESC_SRV1` | **Bounded Context Capacity** | The service enforces a hard cap of `MAX_ACTIVE_CONTEXTS` (1024) concurrent registered contexts to prevent memory exhaustion DoS. |
| `PRIVESC_SRV2` | **Mandatory Pre-Registration** | Actions and elevation requests require an existing, registered actor context (`PRIVESC_ERR_ACTOR_NOT_FOUND`). |
| `PRIVESC_SRV3` | **Strict Transition Gating** | Elevation transitions require an evaluated verdict of `Allowed`. Any request requiring a grant must provide a valid grant ID. |
| `PRIVESC_SRV4` | **Atomic State Transitions** | Context state mutations (`elevate`, `drop`, `revoke`) execute atomically in-memory with zero partial updates. |
| `PRIVESC_SRV5` | **Unprivileged Downgrade Law** | Any active context may voluntarily drop to a lower privilege tier without requiring external authorization. |
| `PRIVESC_SRV6` | **Kernel Tier Immutability** | Any elevation request targeting `PrivilegeLevel::SystemKernel` is rejected unconditionally. |

---

## 3. Core Service API Contract

```rust
pub struct PrivilegeService {
    contexts: HashMap<String, PrivilegeContext>,
    base_levels: HashMap<String, PrivilegeLevel>,
    max_contexts: usize,
}

impl PrivilegeService {
    pub fn new() -> Self;
    pub fn with_capacity(max_contexts: usize) -> Self;
    pub fn register_context(&mut self, context: PrivilegeContext) -> Result<(), String>;
    pub fn unregister_context(&mut self, actor_id: &str) -> Result<PrivilegeContext, String>;
    pub fn get_context(&self, actor_id: &str) -> Option<&PrivilegeContext>;
    pub fn contains_actor(&self, actor_id: &str) -> bool;
    pub fn request_elevation(&mut self, req: PrivilegeTransitionRequest) -> Result<PrivilegeContext, String>;
    pub fn drop_privilege(&mut self, actor_id: &str, target_level: PrivilegeLevel) -> Result<PrivilegeContext, String>;
    pub fn revoke_elevation(&mut self, actor_id: &str) -> Result<PrivilegeContext, String>;
    pub fn check_capability(&self, actor_id: &str, capability: PrivilegeCapability) -> bool;
    pub fn list_actors(&self) -> Vec<String>;
    pub fn get_base_level(&self, actor_id: &str) -> Option<PrivilegeLevel>;
    pub fn active_contexts_count(&self) -> usize;
    pub fn clear(&mut self);
}
```

---

## 4. Error Codes & Bounds
- `PRIVESC_DEFAULT_MAX_ACTIVE_CONTEXTS`: 1024.
- `PRIVESC_MIN_MAX_ACTIVE_CONTEXTS`: 1.
- `PRIVESC_MAX_MAX_ACTIVE_CONTEXTS`: 16384.
- `PRIVESC_ERR_ACTOR_NOT_FOUND`: Actor context not registered in service.
- `PRIVESC_ERR_CAPACITY_EXCEEDED`: Active contexts count reached capacity ceiling.
- `PRIVESC_ERR_CONTEXT_EXISTS`: Actor context already registered.
- `PRIVESC_ERR_INVALID_ACTOR`: Actor identifier is empty or contains control characters.
- `PRIVESC_ERR_UNAUTHORIZED_ELEVATION`: Elevation attempted without grant or failed verdict.
- `PRIVESC_ERR_KERNEL_TIER_IMMUTABLE`: Elevation to SystemKernel rejected.
