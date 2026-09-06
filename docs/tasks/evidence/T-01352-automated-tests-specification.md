# T-01352: Init & Service Supervision - Automated Tests: Specification

## Metadata
- **Task ID:** `T-01352`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Automated Tests Specification
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Scope & Objective

Define the formal contract, test cases, and assertion invariants for the automated integration testing suite of the AIOS Init & Service Supervision subsystem (`code/aiosh-rust/aiosh-core/tests/test_service_automated.rs`).
The suite validates system cohesion across the Service Data Model, Supervision Engine (FSM), Topological Dependency Planner, Configuration Subsystem, CLI, and MCP surfaces.

---

## 2. Specification Criteria (ST1..ST5)

### ST1: Multi-Turn Lifecycle FSM Cohesion & State Invariants
- **Inputs**: A sequence of lifecycle actions across a single service entity:
  1. `start`: Transition from `Inactive` to `Active`.
  2. `reload`: Remains `Active` with updated configuration reload timestamp.
  3. `restart`: Transitions `Active` -> `Activating` -> `Active`.
  4. `stop`: Transitions `Active` -> `Deactivating` -> `Inactive`.
  5. `disable`: Changes startup mode to `Disabled`.
  6. `mask`: Sets startup mode to `Masked` (strictly prohibiting activation).
  7. `start` on masked service: MUST fail immediately with error `ServiceIsMasked`.
  8. `unmask` -> `enable`: Restores service to `Enabled` startup mode.
- **Contract**: Store state transitions MUST strictly follow the finite-state machine rules (`CS2`).

### ST2: Dependency DAG Topological Order & Cycle Detection
- **Inputs**:
  - A graph of 5 interdependent services (`db.service`, `cache.service`, `backend.service`, `api.service`, `frontend.service`).
  - A cyclic dependency graph (`service-a` -> `service-b` -> `service-c` -> `service-a`).
- **Contract**:
  - Invoking `store.plan_service_order("frontend.service")` MUST return a strictly ordered sequence where all dependencies precede dependents (Kahn's algorithm).
  - Invoking on a cyclic graph MUST fail loudly with an explicit error identifying the cycle.

### ST3: Store Persistence & Atomic Recovery
- **Inputs**: A populated `ServiceStore` mutated across multiple turns and written to disk via `save_to_path`.
- **Contract**:
  - Reloading from disk via `load_from_path` produces an identical state.
  - Serialization must be atomic to prevent partial or corrupted writes.

### ST4: Configuration-Governed Quotas & Supervision Boundaries
- **Inputs**: A `ServiceConfig` specifying custom limits (`max_entity_count = 10`, `max_store_size_bytes = 64 * 1024`).
- **Contract**:
  - Registering services beyond the entity count ceiling is rejected.
  - Rejection occurs before memory exhaustion.

### ST5: Filtered Query & Catalog Introspection
- **Inputs**: Service store queried via pattern, state, startup mode, and limit filters.
- **Contract**:
  - Query results match exact filter criteria.
  - Limits cap results deterministically.

---

## 3. Reused vs. New Interfaces

### Reused Interfaces:
- `aiosh_core::service::*` (`ServiceSpec`, `ServiceStatus`, `ServiceAction`, `ServiceStartupMode`, `ServiceState`, `ServiceType`).
- `aiosh_core::service_service::{ServiceStore, ServiceActionReport}`.
- `aiosh_core::service_config::ServiceConfig`.

### New Test Suite Deliverables:
- `code/aiosh-rust/aiosh-core/tests/test_service_automated.rs`:
  - `test_st1_lifecycle_fsm_cohesion_and_masking`
  - `test_st2_dependency_dag_order_and_cycle_detection`
  - `test_st3_store_persistence_and_atomic_recovery`
  - `test_st4_configuration_governed_quotas`
  - `test_st5_filtered_query_and_catalog_introspection`
- Criterion `SS6` registered in `tools/test_service_suites.py`.

---

## 4. Acceptance Criteria Verification
- [x] Inputs, outputs, error paths, and persistence effects defined.
- [x] Invariants `ST1..ST5` formally specified.
- [x] Reused vs new deliverables delineated.
- [x] Spec reviewable without reading implementation.
