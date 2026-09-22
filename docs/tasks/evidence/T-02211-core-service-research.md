# Task Evidence: T-02211 (Grant Lifecycle / core service: Research)

## 1. Executive Summary & Objective
Establishes the foundational research, architectural constraints, authoritative standards, and design decisions for the Grant Lifecycle Core Service (`PepGrantService`, Sub-Epic 2, tasks T-02211..T-02220) within the AIOS Security Kernel & PEP Fabric.

---

## 2. Analysis of Existing Codebase & Prior Art

### Existing Codebase Assets
1. **Grant Lifecycle Data Model (`code/aiosh-rust/aiosh-core/src/pep_grant.rs`)**:
   - Entities: `PepGrant`, `PepGrantState` (Requested, Active, Suspended, Revoked, Expired), `PepGrantConstraints` (`not_before`, `expires_at`, `max_invocations`, `max_bytes`, `max_delegation_depth`), `PepGrantRevocation`, and `PepGrantStore`.
   - Invariants: `PEPGRANT1..PEPGRANT6` covering FSM transitions, hygiene validation, attenuation rights containment, quota tracking, cascade revocation, and atomic serialization.
2. **PEP Decision Service Reference (`code/aiosh-rust/aiosh-core/src/pep_decision_service.rs`)**:
   - Architecture: In-memory hash-indexed registry (`HashMap<String, PepPolicyRule>`) augmented by secondary multi-maps (`by_subject`, `by_action`).
   - Persistence: Bounded file size verification (`MAX_PEP_SERVICE_STORE_SIZE = 10 MiB`), path validation (`validate_pep_service_path`), and atomic rename staging.
3. **Audit Ring Fabric (`code/aiosh-rust/aiosh-mcp/src/main.rs`, `dispatch.rs`)**:
   - Every state-altering call emits an immutable audit row into the local SQLite ring (`audit.db`).

### Authoritative Standards & Prior Art
1. **Object-Capability Systems (KeyKOS, EROS, seL4)**:
   - Grants represent dynamic capabilities.
   - Attenuation must monotonically restrict authority; authority can never be expanded during delegation.
   - Revocation in capability systems is achieved either through revocation tokens or indirection nodes (transitive tree revocation).
2. **Macaroons & Biscuit Decentralized Authorization**:
   - Attenuation caveats: First-party caveats restricting context (temporal bounds, quota usage limits).
   - Delegation tree: Every derived token points to its parent and holds attenuating restrictions.
3. **RFC 7009 (OAuth 2.0 Token Revocation) & RFC 7519 (JSON Web Token)**:
   - Standard temporal claims: `nbf` (not before), `exp` (expiration), `iat` (issued at).
   - Revocation semantics: Revoking a parent token invalidates all derived sub-tokens (cascade invalidation).
4. **AIOS Architectural Decision Record ADR-0035**:
   - Fail-closed security default: missing or unverifiable grants must produce explicit denial.
   - Non-silent failure: all errors returned in structured envelopes.
   - Non-repudiation: all administrative transitions written to SQLite audit log.

---

## 3. Fact vs. Assumption Separation

| Item | Status | Details |
|---|---|---|
| In-memory store model | **FACT** | `PepGrantStore` holds grants; `PepGrantService` coordinates execution, indexes, and cache. |
| Invariants PEPGRANT1..6 | **FACT** | Sub-Epic 1 completed and verified data model invariants; Sub-Epic 2 enforces operational service invariants (`GSVC1..GSVC6`). |
| Secondary Indexing | **FACT** | Secondary indices (`by_subject`, `by_parent`, `by_state`) optimize lookup from $O(N)$ to $O(1)$ and cascade traversal from $O(N^2)$ to $O(K)$. |
| Threading & Concurrency | **ASSUMPTION** | A thread-safe handle pattern (`Arc<RwLock<PepGrantService>>`) or cloneable state struct provides safe cross-thread evaluation without deadlocks. |
| Quota Metering Latency | **ASSUMPTION** | In-memory atomic quota counters avoid disk I/O bottlenecks during hot-path authorization checks, with disk sync flushed upon state changes or periodically. |

---

## 4. Unknowns & Decisions Needed Prior to Implementation
1. **Decision 1: Storage Format Parity**:
   - *Decision*: Maintain JSON serialization parity with `PepGrantStore` so stores saved by `PepGrantService` are 100% interoperable with CLI and MCP utilities.
2. **Decision 2: Sweep Policy for Expired Grants**:
   - *Decision*: Implement `sweep_expired(now_iso: &str) -> usize` which inspects active and suspended grants, transitions expired grants to `PepGrantState::Expired`, updates indices, and returns the count of swept grants.
3. **Decision 3: Quota Consumption Semantics**:
   - *Decision*: Separate grant validation (`evaluate_grant`) from usage recording (`record_usage`), allowing read-only validation without quota consumption, while recording actual resource consumption upon execution completion.

---

## 5. Acceptance Confirmation
- [x] Authoritative literature, code assets, and prior art documented with citations.
- [x] Facts separated from assumptions.
- [x] Architectural decisions and service requirements explicitly stated.
- [x] Zero source code changes made during research phase.
