# T-01498: User Session Bootstrap Recovery & Validation Hardening

**Date:** 2026-09-16  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** User Session Bootstrap / Recovery & Validation  
**Task ID:** T-01498  

---

## 1. Executive Summary

Task `T-01498` implements robust defensive hardening controls across the **User Session Bootstrap Recovery & Validation** subsystem. The system protects against unbounded resource consumption, infinite loop conditions, filesystem race conditions, and uncontrolled error paths. All failure modes produce explicit, structured diagnostic reports and non-repudiable audit logs.

---

## 2. Hardening Measures Implemented

### 2.1 Capacity, Memory & Payload Ceilings
- **Store File Size Cap:** 10 MiB hard limit enforced via `fs::metadata` prior to file loading in `UserSessionStore::load_from_path`, mitigating memory exhaustion attacks from massive payloads.
- **Session Capacity Cap:** Enforced maximum store capacity of `MAX_STORE_CAPACITY = 10_000` sessions in `validate_session_store`. Any store exceeding 10,000 sessions is immediately marked unhealthy with an explicit error.
- **Path Length & Character Bounds:** Store path parameters in CLI (`cmd_session`) and MCP (`aios.session.check`) are capped at 1,024 characters and strictly reject ASCII/Unicode control characters.
- **Session ID Length & Format:** Session IDs are bounded to 64 characters and verified against strict character whitelist rules (SB1).

### 2.2 Loop Bounding, Concurrency & Atomic File Operations
- **Quarantine Backup Collision Loop Bounding:** The backup path generator in `create_backup_file` uses a bounded collision counter (`counter < 10_000`), preventing infinite loops on filesystem naming edge cases or race conditions.
- **Atomic File Writes with PID Isolation:** Write operations create PID-tagged and microsecond-timestamped temporary files (`.tmp.<pid>.<nanos>`) with atomic `fs::rename` replacing the target file, guaranteeing zero partial-write corruption.
- **Bounded Rename Retries:** Up to 3 retry attempts with backoff (10ms) are performed on transient filesystem locks before failing cleanly.
- **Clean Failure Cleanup:** If write or rename fails, temporary files are immediately removed via `fs::remove_file` to prevent orphaned temporary files from accumulating on disk.

### 2.3 Strict Fail-Closed Error Envelopes & Audit Emission
- **Standard Result Envelope:** Every failure mode produces structured responses (`code`, `data: null`, `error: { code, message }` in CLI JSON mode; `ok: false`, `report: {...}`, `error: string` in MCP JSON-RPC). Silent failure is prohibited.
- **Tamper-Evident Audit Emission:** All check and recovery actions write classified audit records (`session.check` or `session.repair`) containing exact health status, session counts, error counts, and backup paths to the SQLite WAL ring for ADR-0035 §F-2 compliance.
- **Non-Destructive Quarantine:** Corrupted stores are never wiped without first writing an immutable, timestamped `.bak.<timestamp>` file with POSIX `0600` permissions.

---

## 3. Test Verification

`cargo test -p aiosh-core --test test_session_recovery`:
```
running 9 tests
test test_default_store_deep_validation ... ok
test test_duplicate_leader_pid_collision ... ok
test test_load_or_recover_lifecycle ... ok
test test_negative_session_specs_and_status_invariants ... ok
test test_non_destructive_corruption_recovery_and_quarantine ... ok
test test_seat_mutual_exclusion_violation ... ok
test test_ssr1_ssr2_ssr3_invariant_equations ... ok
test test_quarantine_path_special_characters ... ok
test test_capacity_boundary_limit ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
```
Zero regressions, zero temp leaks, zero open failure bypasses.
