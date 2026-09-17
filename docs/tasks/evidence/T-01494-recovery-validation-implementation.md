# T-01494: User Session Bootstrap Recovery & Validation Implementation

**Date:** 2026-09-16  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** User Session Bootstrap / Recovery & Validation  
**Task ID:** T-01494  

---

## 1. Implementation Overview

Task `T-01494` delivers the working implementation of the **Recovery & Validation** subsystem for User Session Bootstrap in `code/aiosh-rust/aiosh-core/src/session_recovery.rs`.

---

## 2. Implemented Capabilities

### 2.1 Deep Session Store Validation (`validate_session_store`)
- Capacity bounds checking against `MAX_STORE_CAPACITY = 10_000`.
- Bi-directional spec and status map alignment validation.
- Per-session spec invariant checking via `validate_user_session_spec(spec)` (`SB1..SB5`).
- Per-session status invariant checking via `validate_user_session_status(status)` (`SB1..SB5`).
- Cross-entity consistency (username and UID alignment between spec and status).
- Process tracking consistency: detects duplicate `leader_pid` collisions across non-terminated sessions (`SSR5`).
- Hardware seat mutual exclusion: detects multiple `Foreground` sessions on the same hardware seat (`seat0`, etc.) (`SSR6`).
- Computes `valid_sessions`, `invalid_sessions`, `errors`, `warnings`, `healthy`.
- Satisfies mathematical invariants `SSR1..SSR3`:
  - `valid_sessions + invalid_sessions == total_sessions`
  - `healthy == (errors.is_empty() && invalid_sessions == 0)`
  - `invalid_sessions > 0 => errors.len() >= invalid_sessions`

### 2.2 Non-Destructive Quarantine & Backup (`recover_session_store_with_backup`)
- Generates timestamped backup copy `<path>.bak.<YYYYMMDD_HHMMSS_micros>`.
- Enforces strict POSIX `0600` permissions on the quarantine backup.
- Initializes a fresh, seeded `UserSessionStore` (containing `greeter-seat0`).
- Atomically saves the fresh store with `0600` permissions (`SSR4`).

### 2.3 Automated Recovery Entrypoint (`load_or_recover`)
- Handles missing store files by creating and initializing a fresh seeded store.
- Loads existing stores; if healthy, returns `(service, report, false, None)`.
- If store is corrupted (malformed JSON, I/O error, or validation failure), automatically executes quarantine, initializes a fresh store, and returns `(fresh_service, fresh_report, true, Some(backup_path))`.

---

## 3. Test Verification

`cargo test -p aiosh-core session_recovery`:
```
running 5 tests
test session_recovery::tests::test_seat_mutual_exclusion_violation ... ok
test session_recovery::tests::test_validate_default_store_healthy ... ok
test session_recovery::tests::test_duplicate_leader_pid_detection ... ok
test session_recovery::tests::test_validate_invalid_spec_and_status ... ok
test session_recovery::tests::test_load_or_recover_workflow ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; finished in 0.04s
```
Zero regressions, zero compiler errors.
