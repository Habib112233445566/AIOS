# T-01259: Package Management - Automated Tests: Documentation

## Metadata
- **Task ID:** `T-01259`
- **Subsystem:** `code/aiosh-rust` / `tools`
- **Component:** Package Management / Automated Tests Documentation
- **Status:** Complete
- **Date:** 2026-09-04

---

## 1. Documentation Overview
The Package Management subsystem features an automated integration and end-to-end test suite located in `code/aiosh-rust/aiosh-core/tests/test_package_automated.rs` and integrated into the subsystem master test runner `tools/test_package_suites.py` under criterion `PM6`.

### Test Matrix Criteria (PT1..PT6)
1. **PT1: Transaction Plan Determinism & Reproducibility**:
   - Validates that repeated calls to `store.plan_transaction` with identical inputs produce identical transaction IDs, SHA-256 hashes, delta sizes, and action ordering over 50 consecutive iterations.
   - Verifies that dry-run mode (`dry_run: true`) maintains identical plan hashes and deltas.
2. **PT2: Multi-Step Lifecycle Cohesion**:
   - Tests sequential multi-step package lifecycle progression:
     - Install (`Available` $\to$ `Installed`, size delta $+500\text{ KiB}$)
     - Upgrade (`Upgradable` $\to$ `Installed`, size delta $0$ per in-store upgrade, version updated to 1.2.0)
     - Remove (`Installed` $\to$ `Available`, size delta $-550\text{ KiB}$)
3. **PT3: Dependency Closure Failure Modes**:
   - Asserts rejection when an unsatisfied dependency is missing from the store (`CS3`).
   - Asserts rejection when a dependency exists in the store but is neither installed nor present in the transaction actions.
   - Asserts successful plan creation when both the dependency and dependent package are present in the transaction actions.
   - Asserts rejection when target package is not registered in the store.
4. **PT4: Configuration-Governed Store Bounds**:
   - Validates `PackageConfig` invariants: `PC2` store size bounds [64 KiB .. 100 MiB] and `PC3` entity count bounds [10 .. 100,000].
   - Validates atomic store disk persistence and reload integrity under multi-package loads.
5. **PT5: Transaction Anti-Tamper & Rollback Integrity**:
   - Asserts detection and abortion when transaction total size delta is altered prior to execution (`CS4`).
   - Verifies pristine store state preservation on transaction failure (atomic rollback).
   - Verifies that dry-run execution reports simulated deltas without mutating stored package states.
6. **PT6: Boundary & Negative Matrix**:
   - Asserts rejection of empty action arrays.
   - Asserts rejection when transaction actions exceed 256 entries.
   - Asserts error when unregistering non-existent packages.
   - Asserts rejection of duplicate package registrations (`CS1`).

---

## 2. Invocation & Running Examples

### Running the Subsystem Test Runner (Criteria PM1..PM6)
```bash
python tools/test_package_suites.py
```
Expected output:
```
[+] PM1 package data model integrity & invariants (PM1..PM5)
[+] PM2 package core service integrity & invariants (CS1..CS5)
[+] PM3 package CLI surface commands & options (validate/list/show/search/plan/apply)
[+] PM4 package MCP tool surface (validate/list/get/plan/search/apply)
[+] PM5 package configuration resolution & invariants (PC1..PC6)
[+] PM6 package automated integration test matrix (PT1..PT6)

PASS: package_suites criteria (PM1..PM6)
```

### Running the Automated Integration Test in Isolation
```bash
cargo test -p aiosh-core --test test_package_automated
```
Expected output:
```
running 6 tests
test test_pt1_plan_determinism_and_reproducibility ... ok
test test_pt2_multi_step_lifecycle_cohesion ... ok
test test_pt3_dependency_closure_failure_modes ... ok
test test_pt4_config_governed_store_bounds ... ok
test test_pt5_anti_tamper_and_rollback_integrity ... ok
test test_pt6_boundary_and_negative_matrix ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## 3. Constraints & Known Limitations (Honest)
1. **Transaction Action Upper Bound**: Transaction plans are strictly bounded to at most 256 actions per plan (`actions.len() <= 256`). Batches exceeding this limit are rejected to prevent memory exhaustion and execution timeouts.
2. **Offline Hermetic Testing**: Automated tests run 100% offline and hermetically using synthetic package descriptors and canonical reference sets; they do not require internet access, root/sudo privileges, or live host package managers (`apt`/`apk`).
3. **Entity Count and Store Capacity Bounds**: Store instances are bounded to $[10 \dots 100,000]$ entities and $100\text{ MiB}$ on disk.

---

## 4. Linked Evidence Chain
- Research: `docs/tasks/evidence/T-01251-automated-tests-research.md`
- Specification: `docs/tasks/evidence/T-01252-automated-tests-specification.md`
- Scaffold: `docs/tasks/evidence/T-01253-automated-tests-scaffold.md`
- Implementation: `docs/tasks/evidence/T-01254-automated-tests-implementation.md`
- Unit Test: `docs/tasks/evidence/T-01255-automated-tests-unit-test.md`
- Integration: `docs/tasks/evidence/T-01256-automated-tests-integration.md`
- Security Review: `docs/tasks/evidence/T-01257-automated-tests-security-review.md`
- Hardening: `docs/tasks/evidence/T-01258-automated-tests-hardening.md`
