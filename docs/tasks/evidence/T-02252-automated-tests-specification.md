# Task Evidence: T-02252 (Grant Lifecycle Automated Tests: Specification)

## 1. Metadata
- **Task ID:** `T-02252`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle Automated Tests Specification
- **Status:** Complete
- **Date:** 2026-09-23
- **Author:** AIOS Security Architecture & Verification Team

---

## 2. Test Suite Specification Architecture

This specification formalizes the exact inputs, expected outputs, state transitions, security boundary validations, and error conditions across the 8 automated test vectors (`AUTOGRANT1`..`AUTOGRANT8`).

### 2.1 Interface & Module Boundaries
The test harness reuses existing core domain types without introducing breaking changes or novel production APIs:
- `aiosh_core::pep_grant::{PepGrant, PepGrantState, PepGrantConstraints, PepGrantStore}`
- `aiosh_core::pep_grant_service::{PepGrantService, GSVC_ERR_NOT_FOUND, GSVC_ERR_INVALID_TRANSITION, GSVC_ERR_ATTENUATION}`
- `aiosh_core::pep_grant_config::PepGrantConfig`
- `aiosh_core::capability::{CapabilityScope, CapabilityRight}`
- Test fixture: `MockPepGrantEnv` (using `tempfile::TempDir`)

---

## 3. Detailed Test Vector Specifications

### AUTOGRANT1: Scale & High-Volume Issuance
- **Goal:** Verify that `PepGrantService` efficiently issues, indexes, and queries large volumes of grants with zero degradation.
- **Input:**
  - Ingest $N = 1,000$ synthetic grants into `PepGrantService`.
  - Distribute across 50 subjects (`agent:0`..`agent:49`) with alternating scopes (`Filesystem`, `Network`, `System`).
- **Assertion Criteria:**
  - `service.len() == 1,000`.
  - For each subject $S$, `service.list_grants_for_subject(S).len() == 20`.
  - `service.list_grants_by_state(PepGrantState::Requested).len() == 1,000`.
  - Point lookup by ID executes in $O(1)$ time for every grant.
- **Performance Boundary:**
  - In-memory ingestion and indexing completed in $< 500\text{ ms}$.

### AUTOGRANT2: Multi-Tier Attenuation Hierarchy
- **Goal:** Verify strict monotonic attenuation down an 8-level linear delegation chain.
- **Hierarchy:**
  - Level 0: Root grant issued to `agent:root`, rights = `[Read, Write, Execute, Delete, Admin, Delegate]`, scope = `/var`, `max_delegation_depth = 8`, `delegation_depth = 0`.
  - Levels 1..7: Each level attenuates its parent with narrowed path (`/var/data`, `/var/data/sub`, etc.) and monotonically reduced rights.
- **Assertion Criteria:**
  - Attenuating at Level 7 to create Level 8 fails because `delegation_depth >= max_delegation_depth`.
  - Attempting to add rights at any intermediate tier (e.g. Level 3 requesting `Admin`) returns `Err(GSVC_ERR_ATTENUATION)`.
  - Attempting to broaden filesystem scope (e.g. child requesting `/etc` when parent has `/var`) returns `Err(GSVC_ERR_ATTENUATION)`.
  - Attempting to extend expiration timestamp returns `Err(GSVC_ERR_ATTENUATION)`.

### AUTOGRANT3: Branching Cascade Revocation Invariant
- **Goal:** Verify exact transitive closure revocation across complex branching trees.
- **Topology:**
  - Root: $G_0$
  - Branch A: $G_0 \to G_{A1} \to (G_{A2a}, G_{A2b})$
  - Branch B: $G_0 \to G_{B1} \to (G_{B2a}, G_{B2b})$
- **Execution:**
  - Activate all grants.
  - Revoke $G_{A1}$.
- **Assertion Criteria:**
  - `revoke_grant(G_{A1})` returns exactly set $\{G_{A1}, G_{A2a}, G_{A2b}\}$.
  - Grants $G_{A1}$, $G_{A2a}$, $G_{A2b}$ have state `Revoked`.
  - Grants $G_0$, $G_{B1}$, $G_{B2a}$, $G_{B2b}$ remain in state `Active`.
  - Index `list_grants_by_state(Active)` reflects exactly 4 active grants.

### AUTOGRANT4: Mass Expiration Sweeping Stress
- **Goal:** Verify batch temporal validation and index consistency.
- **Input:**
  - 100 grants configured with `expires_at = Utc::now() - 2 hours`.
  - 100 grants configured with `expires_at = Utc::now() + 2 hours`.
  - All 200 grants transitioned to `Active`.
- **Execution:**
  - Call `service.sweep_expired_grants()`.
- **Assertion Criteria:**
  - Returns `count == 100`.
  - Exactly 100 grants in `Expired` state; exactly 100 grants in `Active` state.
  - Secondary sweep immediately following returns `count == 0` (idempotency).

### AUTOGRANT5: Atomic Persistence & Reload Integrity
- **Goal:** Validate atomic filesystem write semantics and reload fidelity under storage path configuration.
- **Execution:**
  - Initialize `MockPepGrantEnv` with a temporary storage path.
  - Issue 50 grants, transition 25 to `Active`, revoke 10, sweep.
  - Drop the `PepGrantService` memory instance.
  - Re-open a new service pointing to the same storage path.
- **Assertion Criteria:**
  - Reloaded service contains all 50 grants with exact states preserved.
  - Secondary indexes (`by_subject`, `by_state`, `by_parent`) are fully reconstructed.
  - No temporary `.tmp` files left in directory.

### AUTOGRANT6: Adversarial Boundary & Fuzzing
- **Goal:** Validate fail-safe behavior against malicious or corrupted grant parameters.
- **Test Vectors:**
  1. Identifier format violation: contains whitespace, newlines, control characters, or path traversals (`../../`).
  2. Oversized strings: ID $> 64$ characters, subject $> 256$ characters.
  3. Cyclical parent reference: Grant $A$ sets parent $B$, Grant $B$ sets parent $A$.
  4. Quota overflow: requesting negative or non-monotonic invocation/byte quotas.
- **Assertion Criteria:**
  - All vectors rejected cleanly at creation/attenuation time.
  - Zero panics, zero memory leaks, store state unchanged.

### AUTOGRANT7: Concurrent Thread-Safety Pattern
- **Goal:** Confirm safe concurrent read and write operations.
- **Execution:**
  - Wrap `PepGrantService` in `Arc<RwLock<PepGrantService>>`.
  - Spawn 4 reader threads querying grants by subject and checking active state.
  - Spawn 2 writer threads issuing new grants and attenuating.
- **Assertion Criteria:**
  - All threads join successfully without deadlock or panics.
  - Final store count equals expected initial + writer inserts.

### AUTOGRANT8: Cross-Substrate Interoperability Contract
- **Goal:** Verify that serialized JSON schema matches expectations across CLI, MCP, and Core.
- **Execution:**
  - Serialize full `PepGrantStore` containing diverse grants.
  - Parse as generic `serde_json::Value` and verify canonical fields: `id`, `issuer`, `subject`, `scope`, `rights`, `state`, `constraints`, `delegation_depth`, `max_delegation_depth`.
- **Assertion Criteria:**
  - All expected keys present with standard data types.
  - Validates full roundtrip serialization/deserialization.

---

## 4. Verification & Acceptance Criteria
- [x] All 8 test vectors specified with inputs, execution paths, and assertions.
- [x] Zero changes to production code interfaces required.
- [x] Error codes and failure paths explicitly defined.
