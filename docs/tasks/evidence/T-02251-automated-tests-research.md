# Task Evidence: T-02251 (Grant Lifecycle Automated Tests: Research)

## 1. Metadata
- **Task ID:** `T-02251`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle Automated Tests Research (Sub-Epic 6 Launch)
- **Status:** Complete
- **Date:** 2026-09-23
- **Author:** AIOS Security Architecture & Verification Team

---

## 2. Objective & Scope
The goal of Sub-Epic 6 (`T-02251` through `T-02260`) is to construct, verify, and document an industrial-grade automated test suite for the **PEP Grant Lifecycle** subsystem.
This research task establishes authoritative facts, constraints, security invariants, and prior art required to formulate test specifications (`T-02252`), scaffolds (`T-02253`), and test implementations (`T-02254`..`T-02255`).

---

## 3. Authoritative Sources & Prior Art

### 3.1 Security & Capability Models
1. **Saltzer and Schroeder (1975)** — *The Protection of Information in Computer Systems*:
   - *Economy of Mechanism*: Automated testing must verify that the grant state machine transitions are minimal, deterministic, and complete.
   - *Fail-safe Defaults*: Any invalid grant identifier, out-of-order state transition, or revoked ancestor must default to rejection.
   - *Complete Mediation*: Every capability evaluation check must traverse active grant validity and verify uninterrupted lineage up to the root.
2. **Macaroons: Cookies with Contextual Caveats (Birgisson et al., 2014)**:
   - Verifies monotonic attenuation: a child grant can never acquire rights, widen scopes, or expand validity intervals beyond those granted to its parent.
3. **OAuth 2.0 Token Exchange (RFC 8693) & Hierarchical Delegations**:
   - Delegation chains require strict recursion depth bounds to prevent resource exhaustion (stack overflow / cycle traversal denial-of-service).
4. **POSIX Atomic Replacement (`rename(2)`) & Crash Invariants**:
   - `write-to-tempfile -> fsync -> rename` ensures atomic persistence where a process crash or sudden failure leaves either the clean prior state or the fully written new state, never a partially truncated JSON payload.

### 3.2 AIOS Codebase Precedents
1. **`test_capability_automated.rs` (`T-02055`)**:
   - Established the `MockCapabilityEnv` pattern leveraging `tempfile::TempDir` for hermetic testing.
   - Test matrix structured around formal invariant codes (`CAPTEST1..CAPTEST6`).
2. **`code/aiosh-rust/aiosh-core/src/pep_grant.rs`**:
   - Implements `PepGrant`, `PepGrantState`, `PepGrantConstraints`, `PepGrantStore`, attenuation logic, and cascade revocation.
3. **`code/aiosh-rust/aiosh-core/src/pep_grant_service.rs`**:
   - High-level coordinator holding in-memory store, multi-attribute indexing (`by_subject`, `by_issuer`, `by_state`, `by_parent`), and sync persistence.
4. **`code/aiosh-rust/aiosh-core/src/pep_grant_config.rs`**:
   - Configuration management, bounds enforcement (`max_store_bytes`, `default_max_delegation_depth`), and environment mapping.

---

## 4. Facts vs. Assumptions

| Item | Status | Citation / Observation |
|---|---|---|
| **F-01: FSM State Space** | **Fact** | States are `Requested`, `Active`, `Revoked`, `Expired`. Terminal states `Revoked` and `Expired` prohibit further transitions. |
| **F-02: Monotonic Attenuation** | **Fact** | `pep_grant::attenuate_grant` enforces strict rights subset, path prefix narrowing, time interval tightening, and quota clamping. |
| **F-03: Cascade Revocation** | **Fact** | Revoking grant $G$ must recursively mark all transitive descendants of $G$ as `Revoked`. |
| **F-04: Expiration Sweeping** | **Fact** | `sweep_expired_grants` transitions expired active grants to `Expired` and updates indexed lookup sets. |
| **F-05: Atomic Persistence** | **Fact** | `PepGrantStore::save_to_disk` writes to a sibling `.tmp` file, calls sync, and replaces the target via atomic filesystem rename. |
| **A-01: Scale Target** | **Assumption** | Testing at 1,000+ grants per synthetic batch validates sub-millisecond query performance and acceptable memory footprints for userspace PEP daemon. |
| **A-02: Deep Hierarchy Bound** | **Assumption** | Maximum delegation depth of 8 (configurable up to 32) prevents cyclic or unbounded graph traversals without stack overflow. |
| **A-03: Concurrency Safety** | **Assumption** | Although `PepGrantService` uses internal `RwLock`/`Mutex` or interior mutability when wrapped, automated tests must verify thread-safe usage patterns. |

---

## 5. Identified Automated Test Vectors (`AUTOGRANT1`..`AUTOGRANT8`)

The automated test harness will be structured across eight distinct verification categories:

1. **`AUTOGRANT1: Scale & High-Volume Issuance`**:
   - Issue 1,000 synthetic grants with diverse subjects and scopes.
   - Verify indexing integrity, O(1) ID lookups, and sub-millisecond query time.
2. **`AUTOGRANT2: Multi-Tier Attenuation Hierarchy`**:
   - Build an 8-level linear delegation chain (Root -> T1 -> ... -> T7).
   - Verify monotonicity of rights, scopes, and expiration bounds at each tier.
3. **`AUTOGRANT3: Transitive Cascade Revocation Invariant`**:
   - Construct branching delegation trees (binary trees, star graphs).
   - Revoke root/intermediate nodes and assert all descendants are revoked while peer branches remain active.
4. **`AUTOGRANT4: Mass Expiration Sweeping Stress`**:
   - Ingest mixed populations of current and pre-expired grants.
   - Trigger sweep; verify state transitions, index updates, and zero false positives.
5. **`AUTOGRANT5: Atomic Persistence & Recovery Resilience`**:
   - Stress write operations with rapid mutations and verify store reload fidelity and temporary file cleanup.
6. **`AUTOGRANT6: Adversarial & Boundary Fuzzing`**:
   - Test depth limit overruns (> 32), cyclic parent links, path traversal strings (`../`), oversized payloads (> 50MB), and invalid characters.
7. **`AUTOGRANT7: Concurrent Evaluation & Read/Write Safety`**:
   - Test concurrent evaluation readers alongside background sweeping and issuance using `Arc<RwLock<PepGrantService>>`.
8. **`AUTOGRANT8: CLI and MCP Interoperability Harness`**:
   - Validate that both CLI subcommand outputs and MCP JSON-RPC handlers deserialize and mutate the shared grant store seamlessly.

---

## 6. Unknowns & Resolved Decisions

1. **Test Harness Architecture**:
   - **Decision:** Introduce `MockPepGrantEnv` in `code/aiosh-rust/aiosh-core/tests/test_pep_grant_automated.rs` using `tempfile::TempDir`, mirrors `MockCapabilityEnv`.
2. **No New Dependencies**:
   - **Decision:** Use only standard library, `tempfile`, `chrono`, and existing workspace dependencies (`serde`, `serde_json`). No external benchmarking or fuzzing frameworks required for standard automated tests.
3. **Deterministic Timestamps**:
   - **Decision:** Tests use `Utc::now() +/- Duration::hours/minutes` to guarantee test reproducibility across test runners.

---

## 7. Acceptance Criteria Checklist
- [x] Authoritative sources and security models collected and cited.
- [x] Fact vs assumption table documented.
- [x] Concrete test vector codes `AUTOGRANT1..AUTOGRANT8` formulated.
- [x] Zero code modifications during Research phase.
