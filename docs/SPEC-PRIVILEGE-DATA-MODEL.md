# Specification: Privilege Escalation Prevention Data Model

## 1. Overview
The Privilege Escalation Prevention Data Model (`PRIVESC1`..`PRIVESC6`) establishes the formal domain structures, privilege levels, typed capabilities, and transition request evaluation rules governing privilege management within AIOS userspace and security kernel boundaries.

---

## 2. Invariants & Rulesets

| Invariant | Name | Description |
|---|---|---|
| `PRIVESC1` | **Monotonic Escalation Law** | Any privilege escalation where `target_level > from_level` mandates an explicit, cryptographically verifiable PEP authorization grant token. |
| `PRIVESC2` | **Kernel Tier Immutability** | Transitions targeting `PrivilegeLevel::SystemKernel` from userspace are unconditionally denied (`PRIVESC_ERR_KERNEL_TIER_IMMUTABLE`), regardless of grant status. |
| `PRIVESC3` | **Safe Downgrade Principle** | Privilege de-escalation (`drop_to_level`) is unprivileged and immediately strips capabilities requiring higher tiers, wiping active elevation tokens. |
| `PRIVESC4` | **Input & Bounds Defense** | Actor identifiers are capped at `MAX_ACTOR_ID_LEN` (128 bytes), grants at `MAX_GRANT_ID_LEN` (256 bytes), and capability sets at `MAX_CAPABILITIES_COUNT` (32). Control characters are rejected. |
| `PRIVESC5` | **Pre-Flight Validation** | All transition requests must pass structural schema validation before verdict evaluation. |
| `PRIVESC6` | **Audit Provenance** | Contexts track `actor_id`, `elevation_grant_id`, and `session_id` to enable tamper-evident audit trail correlation. |

---

## 3. Data Structures & Hierarchy

### Discrete Privilege Levels
```rust
pub enum PrivilegeLevel {
    Guest = 0,
    User = 10,
    Operator = 20,
    Admin = 30,
    SystemKernel = 40,
}
```

### Typed Privilege Capabilities
- `NetworkConnect` (Floor: `Guest`)
- `ProcessSpawn` (Floor: `User`)
- `FilesystemWrite` (Floor: `User`)
- `NetworkListen` (Floor: `Operator`)
- `MemoryInspect` (Floor: `Operator`)
- `AuditLogAdmin` (Floor: `Admin`)
- `SystemReboot` (Floor: `Admin`)
- `KernelModuleLoad` (Floor: `Admin`)

### Verdict Outcomes
- `PrivilegeEscalationVerdict::Allowed`
- `PrivilegeEscalationVerdict::GrantRequired { reason: String }`
- `PrivilegeEscalationVerdict::Denied { reason: String }`

---

## 4. Error Codes
- `PRIVESC_ERR_INVALID_ACTOR`: Actor ID is empty, whitespace-only, contains control characters, or exceeds 128 bytes.
- `PRIVESC_ERR_INVALID_GRANT`: Grant ID contains control characters or exceeds 256 bytes.
- `PRIVESC_ERR_KERNEL_TIER_IMMUTABLE`: Transition to SystemKernel requested from userspace.
- `PRIVESC_ERR_CAPABILITY_OVERFLOW`: Capability count exceeds 32.
- `PRIVESC_ERR_CAPABILITY_UNAUTHORIZED`: Capability requires higher tier than target without grant.
- `PRIVESC_ERR_UNAUTHORIZED_ELEVATION`: Elevation attempted without valid PEP authorization grant.
