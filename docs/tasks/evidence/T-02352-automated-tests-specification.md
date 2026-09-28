# Task Evidence: T-02352 (Audit Chain Extensions / automated tests: Specification)

## 1. Metadata
- **Task ID:** `T-02352`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions Automated Tests Specification
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 6: Automated Tests (2/10) — Specification

---

## 2. Specification of Automated Test Vectors

### 2.1 Test Harness & Fixtures
- Harness module: `code/aiosh-rust/aiosh-core/tests/test_audit_chain_automated.rs`
- Backing environment: Hermetic isolated temporary directories created with `tempfile::tempdir`.
- Engine: `AuditChainService` backed by file-backed SQLite database in WAL mode.

### 2.2 Formal Test Vectors

#### Vector 1: `test_autoaudit1_high_volume_scale`
- **Input:** 1,000 extended audit events appended in sequence with synthesized provenance, causal references, and metadata.
- **Verification:**
  - Ingestion completes with zero errors.
  - `service.verify_integrity()` returns `VerifyResult { ok: true, rows_checked: 1000, corrupted_row_id: None, ... }`.
  - Point lookup by SHA-256 hash succeeds across randomly selected rows.

#### Vector 2: `test_autoaudit2_deep_causal_lineage`
- **Input:** A chain of 30 sequential events, where each event $E_i$ declares $E_{i-1}$ as its parent in `causal_links`.
- **Verification:**
  - `service.trace_ancestry(E_30, 30)` traces back to $E_1$, returning all 29 ancestors in exact topological sequence.
  - `max_depth_reached` remains `false` when within bound.

#### Vector 3: `test_autoaudit3_branching_diamond_dag`
- **Input:** Diamond graph: Root $A \to$ Branches $B$ and $C \to$ Merged event $D$ referencing both $B$ and $C$ as causal parents.
- **Verification:**
  - `service.trace_ancestry(D, 10)` returns $B$, $C$, and root $A$ without duplication.
  - Deduping ensures visited nodes are processed exactly once.

#### Vector 4: `test_autoaudit4_cycle_detection_immunity`
- **Input:** Corrupted graph containing deliberate circular causality ($A \to B \to A$).
- **Verification:**
  - `trace_ancestry` executes in linear time without infinite recursion or stack overflow.
  - Returns graceful report with visited nodes.

#### Vector 5: `test_autoaudit5_cryptographic_signatures`
- **Input:**
  - Event with valid Ed25519 signature envelope.
  - Event with tampered signature bits.
  - Event without signature (`None`).
- **Verification:**
  - Valid signature yields `is_valid: true`, `has_signature: true`.
  - Tampered signature yields `is_valid: false`, `has_signature: true`.
  - Unsigned event yields `is_valid: false`, `has_signature: false`.

#### Vector 6: `test_autoaudit6_multi_column_query_clamping`
- **Input:** Filter combinations across session, trace, actor, tool, and parent hash.
- **Verification:**
  - Accurate query filtering results.
  - Query limit clamps to configured bound.

#### Vector 7: `test_autoaudit7_concurrency_safety`
- **Input:** 4 reader threads continuously querying events while 2 writer threads concurrently append events to the ring.
- **Verification:**
  - Zero deadlocks, zero panics, zero database lock errors.
  - Integrity of the resulting audit chain remains 100% verified.

#### Vector 8: `test_autoaudit8_legacy_parity`
- **Input:** Ring initialized with legacy unextended audit rows, subsequently appended with extended rows.
- **Verification:**
  - `verify_integrity()` validates both legacy and extended rows without breaking SHA-256 chain continuity.

---

## 3. Acceptance Confirmation
- [x] Inputs, outputs, error conditions, and state transitions fully specified.
- [x] All 8 formal test vectors (`AUTOAUDIT1..AUTOAUDIT8`) documented.
- [x] Concurrency, scale, and adversarial cases covered.
