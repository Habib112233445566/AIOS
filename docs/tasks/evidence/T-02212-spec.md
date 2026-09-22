# Task Evidence: T-02212 (Grant Lifecycle / core service: Specification)

## 1. Specification Overview
Defines the authoritative operational contract, API signatures, error codes, invariants, and persistence semantics for `PepGrantService` (`code/aiosh-rust/aiosh-core/src/pep_grant_service.rs`) in the AIOS Security Kernel & PEP Fabric.

---

## 2. Core Service Invariants (`GSVC1..GSVC6`)

| Invariant | Name | Formal Rule |
|---|---|---|
| **`GSVC1`** | **Multi-Index Consistency** | Primary map (`grants: HashMap<String, PepGrant>`) and secondary indices (`by_subject`, `by_parent`, `by_state`) must remain strictly synchronized. Any insertion, state transition, or removal atomically updates all corresponding secondary index entries. |
| **`GSVC2`** | **Authoritative State Transition Engine** | State transitions strictly enforce the PEP Grant FSM (`Requested -> Active/Revoked`, `Active -> Suspended/Revoked/Expired`, `Suspended -> Active/Revoked/Expired`, terminal sinks `Revoked` and `Expired`). Illegal transitions are rejected with `GSVC_ERR_INVALID_TRANSITION`. |
| **`GSVC3`** | **Monotonic Attenuation & Delegation** | `attenuate_grant()` requires parent to be `Active`, possess `CapabilityRight::Delegate`, and have `max_delegation_depth > 0`. Child rights must be a subset of parent rights. Child inherits parent expiration boundary and decremented delegation depth. Rights escalation fails with `GSVC_ERR_ATTENUATION`. |
| **`GSVC4`** | **Action Authorization & Quota Verification** | `evaluate_grant()` evaluates grant eligibility for a specific subject, required right, and current timestamp. Verifies temporal bounds (`not_before <= now < expires_at`) and invocation/byte quota exhaustion. Usage is metered via `record_grant_usage()`. |
| **`GSVC5`** | **Atomic Cascade Revocation** | `revoke_grant(..., cascade=true)` computes the transitive closure over `by_parent` DAG and transitions the target grant and all descendants to `Revoked`. Operator identification, revocation timestamp, and audit reason are recorded on each revoked entity. |
| **`GSVC6`** | **Expiration Sweep & Atomic Persistence** | `sweep_expired()` identifies all active/suspended grants that have passed `expires_at` or exhausted quotas, transitioning them to `Expired` and pruning active indexes. Persistence enforces atomic replace staging (`.tmp.<pid>.<nonce>`), directory checks, and a 10 MiB payload ceiling. |

---

## 3. Public API Contract & Signatures

```rust
// Constants and error identifiers
pub const MAX_GRANTS_IN_SERVICE: usize = 5000;
pub const MAX_GRANT_SERVICE_STORE_SIZE: u64 = 10 * 1024 * 1024; // 10 MiB

pub const GSVC_ERR_CAPACITY: &str = "GSVC_ERR_CAPACITY";
pub const GSVC_ERR_NOT_FOUND: &str = "GSVC_ERR_NOT_FOUND";
pub const GSVC_ERR_INVALID_TRANSITION: &str = "GSVC_ERR_INVALID_TRANSITION";
pub const GSVC_ERR_ATTENUATION: &str = "GSVC_ERR_ATTENUATION";
pub const GSVC_ERR_QUOTA_EXCEEDED: &str = "GSVC_ERR_QUOTA_EXCEEDED";
pub const GSVC_ERR_EXPIRED: &str = "GSVC_ERR_EXPIRED";
pub const GSVC_ERR_NOT_YET_VALID: &str = "GSVC_ERR_NOT_YET_VALID";
pub const GSVC_ERR_VALIDATION: &str = "GSVC_ERR_VALIDATION";
pub const GSVC_ERR_IO: &str = "GSVC_ERR_IO";

pub struct PepGrantService {
    grants: HashMap<String, PepGrant>,
    by_subject: HashMap<String, HashSet<String>>,
    by_parent: HashMap<String, HashSet<String>>,
    by_state: HashMap<PepGrantState, HashSet<String>>,
    storage_path: Option<PathBuf>,
}

impl PepGrantService {
    pub fn new() -> Self;
    pub fn with_storage_path(self, path: PathBuf) -> Self;
    pub fn storage_path(&self) -> Option<&Path>;

    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;

    // CRUD & Lifecycle
    pub fn issue_grant(&mut self, grant: PepGrant) -> Result<(), String>;
    pub fn get_grant(&self, id: &str) -> Option<&PepGrant>;
    pub fn list_grants(&self) -> Vec<PepGrant>;
    pub fn list_grants_for_subject(&self, subject: &str) -> Vec<&PepGrant>;
    pub fn list_grants_by_state(&self, state: PepGrantState) -> Vec<&PepGrant>;
    pub fn transition_grant(&mut self, id: &str, target_state: PepGrantState) -> Result<(), String>;

    // Attenuation & Delegation
    pub fn attenuate_grant(
        &mut self,
        parent_id: &str,
        child_id: &str,
        child_subject: &str,
        delegated_rights: Vec<CapabilityRight>,
    ) -> Result<PepGrant, String>;

    // Evaluation & Quota Tracking
    pub fn evaluate_grant(
        &self,
        grant_id: &str,
        subject: &str,
        right: CapabilityRight,
        now_iso: &str,
    ) -> Result<(), String>;

    pub fn record_grant_usage(&mut self, grant_id: &str, bytes: u64) -> Result<(), String>;

    // Revocation & Sweep
    pub fn revoke_grant(
        &mut self,
        id: &str,
        revoked_by: &str,
        reason: &str,
        cascade: bool,
    ) -> Result<usize, String>;

    pub fn sweep_expired(&mut self, now_iso: &str) -> Result<usize, String>;

    // Persistence
    pub fn save_to_path(&self, path: &Path) -> Result<(), String>;
    pub fn load_from_path(path: &Path) -> Result<Self, String>;
    pub fn sync(&self) -> Result<(), String>;
}
```

---

## 4. Reused vs. New Interfaces
- **Reused Interfaces**:
  - `PepGrant`, `PepGrantState`, `PepGrantConstraints`, `PepGrantRevocation` from `aiosh_core::pep_grant`.
  - `CapabilityRight`, `CapabilityScope`, `validate_identifier`, `validate_scope` from `aiosh_core::capability`.
  - `validate_pep_service_path` pattern from `aiosh_core::pep_decision_service`.
- **New AIOS-Specific Interfaces**:
  - `PepGrantService`: Integrated multi-index lifecycle coordinator.
  - `sweep_expired`: Dynamic cron/on-demand expiration sweeper.
  - Secondary indexing caches (`by_subject`, `by_parent`, `by_state`).

---

## 5. Acceptance Confirmation
- [x] Contract fully specified covering happy paths, failure paths, and audit effects.
- [x] Invariants `GSVC1..GSVC6` mathematically and behaviorally defined.
- [x] Error codes and persistence semantics explicitly detailed.
- [x] Specification reviewable without reading the implementation.
