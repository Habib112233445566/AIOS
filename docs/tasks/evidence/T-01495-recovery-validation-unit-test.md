# T-01495: User Session Bootstrap Recovery & Validation Unit Test

**Date:** 2026-09-16  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** User Session Bootstrap / Recovery & Validation  
**Task ID:** T-01495  

---

## 1. Test Suite Summary

Task `T-01495` delivers comprehensive, focused unit and integration tests for the **User Session Bootstrap Recovery & Validation** subsystem in [`code/aiosh-rust/aiosh-core/tests/test_session_recovery.rs`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/code/aiosh-rust/aiosh-core/tests/test_session_recovery.rs).

The test suite systematically asserts positive paths, deep invalid input permutations, boundary values, collision arbitration, and self-healing lifecycle transitions.

---

## 2. Test Coverage & Invariant Matrix

| Test Function | Target Scope & Invariants | Input Class | Asserted Behavior |
|---|---|---|---|
| `test_ssr1_ssr2_ssr3_invariant_equations` | Invariants `SSR1`, `SSR2`, `SSR3` | Negative & Positive Reports | Rejects mismatched sums (`valid + invalid != total`), hidden errors when healthy, and undercounted errors. Valid reports pass. |
| `test_default_store_deep_validation` | Baseline Canonical State | Positive Valid | Fresh empty store produces `healthy: true`, 0 errors, 0 invalid sessions, and exact valid session match. |
| `test_negative_session_specs_and_status_invariants` | SB1..SB5 Spec & Status Invariants | Negative Invalid | Asserts detection of illegal usernames (uppercase/symbols), missing display on X11 sessions, key/session ID mismatch, and orphaned specs. |
| `test_seat_mutual_exclusion_violation` | Hardware Seat Arbitration (`SSR5`) | Negative Collision | Detects concurrent `Foreground` sessions on `seat0`; raises validation error and advises seat arbitration warning. |
| `test_duplicate_leader_pid_collision` | Process Tracking (`SSR5`) | Negative Collision | Identifies duplicate active process leaders across distinct sessions (`leader_pid = 4040`) and fails health check. |
| `test_capacity_boundary_limit` | Bounds Enforcement | Boundary Limit | Stores with > 10,000 sessions fail capacity threshold checks (`MAX_STORE_CAPACITY`). |
| `test_non_destructive_corruption_recovery_and_quarantine` | Quarantine & Backup (`SSR4`) | Corrupted Payload | Verifies unreadable store files are backed up to `<path>.bak.<timestamp>` with exact corrupted bytes preserved, while fresh store is regenerated. |
| `test_load_or_recover_lifecycle` | Multi-phase Self-Healing Entrypoint | End-to-End Lifecycle | Validates absent file creation, healthy loading without recovery, corrupted JSON quarantine, and semantic corruption quarantine. |
| `test_quarantine_path_special_characters` | Filesystem Hardening | Path Boundaries | Stores in paths with spaces and non-standard characters quarantine and reconstitute cleanly without errors. |

---

## 3. Standalone Verification Output

Executed `cargo test -p aiosh-core --test test_session_recovery`:

```
   Compiling aiosh-core v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.53s
     Running tests\test_session_recovery.rs (target\debug\deps\test_session_recovery-0625e19de184e8a6.exe)

running 9 tests
test test_default_store_deep_validation ... ok
test test_duplicate_leader_pid_collision ... ok
test test_negative_session_specs_and_status_invariants ... ok
test test_non_destructive_corruption_recovery_and_quarantine ... ok
test test_load_or_recover_lifecycle ... ok
test test_quarantine_path_special_characters ... ok
test test_seat_mutual_exclusion_violation ... ok
test test_ssr1_ssr2_ssr3_invariant_equations ... ok
test test_capacity_boundary_limit ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
```

All 9 tests passed with 0 failures, 0 regressions.
