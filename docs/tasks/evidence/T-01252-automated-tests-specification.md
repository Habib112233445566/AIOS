# T-01252: Package Management - Automated Tests: Specification

## Metadata
- **Task ID:** `T-01252`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Package Management / Automated Tests
- **Status:** Complete
- **Date:** 2026-09-04

---

## 1. Scope & Objective
Define the formal contract for the automated integration testing suite of the AIOS Package Management subsystem (`code/aiosh-rust/aiosh-core/tests/test_package_automated.rs`).
The suite validates system cohesion across the Data Model, Core Service, Configuration, CLI, and MCP surfaces.

---

## 2. Specification Criteria (PT1..PT5)

### PT1: Transaction Plan Determinism & Reproducibility
- **Inputs**: A fixed set of package specs and an ordered list of `PackageAction` requests.
- **Contract**: Invoking `store.plan_transaction(&actions)` 50 consecutive times on an identical store snapshot MUST produce identical `PackageTransaction` structures, bit-for-bit identical `plan_hash` values, and identical estimated size deltas.
- **Failure Mode**: Any variance in action ordering, hash output, or sizing is an immediate test failure.

### PT2: Multi-Step Lifecycle Cohesion
- **Inputs**: A sequence of distinct transaction applications across a single package entity:
  1. `Install`: Transition from `PackageState::Uninstalled` to `PackageState::Installed`.
  2. `Upgrade`: Transition from version `v1` to `v2` with associated dependency and size updates.
  3. `Remove`: Transition back from `PackageState::Installed` to `PackageState::Uninstalled`.
- **Contract**: Store state transitions MUST update cleanly at each step, persisting accurate `PackageState`, updating package catalogs, and accurately calculating installed size deltas.

### PT3: Dependency Closure Failure Modes & Negative Bounds
- **Inputs**:
  - A package requesting an unsatisfied dependency (missing prerequisite).
  - Cyclic dependency graphs (e.g. `pkg-alpha` depends on `pkg-beta`, and `pkg-beta` depends on `pkg-alpha`).
  - Cross-format incompatible package actions.
- **Contract**:
  - `store.plan_transaction` MUST return an explicit error (`PlanError::MissingDependency` or `PlanError::CyclicDependency`) and reject plan creation.
  - Negative test assertions must verify that no intermediate state or partial transactions are recorded.

### PT4: Configuration-Governed Store Bounds Enforcement
- **Inputs**: A `PackageConfig` specifying custom limits (`max_entity_count = 10`, `max_store_size_bytes = 64 * 1024`).
- **Contract**:
  - Attempting to register packages beyond the configured ceiling MUST be rejected before memory allocation or disk serialization.
  - Store file serialization exceeding the configured maximum size MUST return an explicit error and leave existing persisted data untouched.

### PT5: Transaction Anti-Tamper & Rollback Integrity
- **Inputs**:
  - A validly generated `PackageTransaction` whose actions or hash have been altered after planning.
  - A transaction executed with `dry_run = true`.
- **Contract**:
  - `store.execute_transaction` MUST recompute the SHA-256 plan hash over actions. If mismatched, it MUST reject execution with `ExecutionError::TamperedTransaction` and leave all package states unchanged.
  - If `dry_run = true`, the method returns projected changes without mutating internal store state.

---

## 3. Reused vs. New Interfaces
- **Reused Interfaces**:
  - `aiosh_core::package::*` (`PackageSpec`, `PackageFormat`, `PackageState`, `PackageDependency`, `PackageAction`, `PackageTransaction`, `PackageQuery`).
  - `aiosh_core::package_service::PackageStore`.
  - `aiosh_core::package_config::PackageConfig`.
- **New Interfaces / Test Deliverable**:
  - `code/aiosh-rust/aiosh-core/tests/test_package_automated.rs` containing test cases:
    - `test_pt1_plan_determinism_and_reproducibility`
    - `test_pt2_multi_step_lifecycle_cohesion`
    - `test_pt3_dependency_closure_failure_modes`
    - `test_pt4_config_governed_store_bounds`
    - `test_pt5_anti_tamper_and_rollback_integrity`
  - Criterion `PM6` registered in `tools/test_package_suites.py`.
