# T-01395: Init & Service Supervision Recovery & Validation Unit Test

**Date:** 2026-09-09  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Init & Service Supervision / Recovery & Validation  
**Task ID:** T-01395  

---

## 1. Executive Summary
Task `T-01395` authored a comprehensive, dedicated automated test suite in `code/aiosh-rust/aiosh-core/tests/test_service_recovery.rs`. The test suite covers positive happy paths, negative syntax and traversal rejections, cyclic dependency detection, mathematical invariant equations `SR1..SR3`, non-destructive timestamped quarantine of corrupted store files, and self-healing canonical store reconstitution.

---

## 2. Test Coverage Matrix

| Test Function | Target Subsystem / Behavior | Invariants Tested | Verification Type |
|---|---|---|---|
| `test_sr1_sr2_sr3_invariant_equations` | Mathematical conservation and consistency checks | `SR1`, `SR2`, `SR3` | Synthetic edge cases & equations |
| `test_default_store_deep_validation` | Canonical default service store validation | `SR1..SR5`, `SS1..SS5` | Production baseline audit |
| `test_negative_service_specs_and_status_invariants` | Name syntax, working_dir traversal, key mismatch, orphan status | `SR1..SR3`, `SS1`, `SS2` | Negative rejection & error attribution |
| `test_dependency_cycle_detection_in_store` | Transitive circular dependency detection via Kahn's algorithm | `CS3`, `SR2` | Graph cycle detection |
| `test_non_destructive_corruption_recovery_and_quarantine` | Corrupt JSON file quarantine to `.corrupt.<ts>.bak` and reconstitution | `SR4`, `SR5` | Filesystem forensics & healing |
| `test_load_or_recover_workflow` | High-level load, validate, and auto-recover state flow | `SR1..SR5` | Lifecycle multi-phase state machine |

---

## 3. Standalone Execution Output

Running `cargo test --test test_service_recovery`:
```text
running 6 tests
test test_negative_service_specs_and_status_invariants ... ok
test test_default_store_deep_validation ... ok
test test_dependency_cycle_detection_in_store ... ok
test test_sr1_sr2_sr3_invariant_equations ... ok
test test_non_destructive_corruption_recovery_and_quarantine ... ok
test test_load_or_recover_workflow ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

All 6 focused unit tests run cleanly standalone, asserting negative rejection and failure modes without regressions.
