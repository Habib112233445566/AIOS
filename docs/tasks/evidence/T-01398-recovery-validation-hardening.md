# T-01398: Init & Service Supervision Recovery & Validation Hardening

**Date:** 2026-09-09  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Init & Service Supervision / Recovery & Validation  
**Task ID:** T-01398  

---

## 1. Executive Summary
Task `T-01398` implemented defensive hardening controls across the **Init & Service Supervision Recovery & Validation** subsystem. The system protects against unbounded resource consumption, filesystem race conditions, and uncontrolled loop iterations during quarantine creation, ensuring fail-closed error handling and non-repudiable audit logging across all error pathways.

---

## 2. Hardening Measures Implemented

### 1. Capacity & Payload Ceilings
- **Store File Ceiling:** 10 MiB limit enforced when reading files in `ServiceStore::load_from_path`, preventing memory exhaustion from oversized files.
- **Service Capacity Ceiling:** Maximum of 10,000 services enforced in `validate_service_store`. Stores exceeding this ceiling trigger immediate non-compliant errors.
- **Path Length & Character Boundaries:** Store path arguments are bounded to 1,024 characters and checked for control characters across CLI (`cmd_service`) and MCP (`call_tool`) surfaces.

### 2. Loop Bounding & Race Resilience
- **Quarantine Collision Protection:** The backup filename collision loop in `create_backup_file` is bounded to a maximum of 10,000 iterations (`while backup_path.exists() && counter < 10_000`), preventing infinite loops on filesystem naming edge cases.
- **PID-Isolated Atomic Tempfiles:** Write operations create PID-tagged temporary files (`.tmp.<pid>`) with atomic `fs::rename` replacing the target file, ensuring zero partial-write corruption even during abrupt power loss.
- **Clean Failure Cleanup:** If a write or rename operation fails, any lingering temporary file is removed (`std::fs::remove_file`) to prevent temp storage accumulation.

### 3. Fail-Closed Error Reporting & Audit Enforcement
- In audit mode (`--fix` absent or `auto_recover: false`), corrupt or unreadable files produce structured diagnostic reports with exit code 1 (`ok: false`), preventing damaged state from executing.
- In recovery mode (`--fix` or `auto_recover: true`), damaged files are non-destructively preserved with timestamped quarantine backups (`.corrupt.<ts>.bak`), and an explicit `service.repair` audit row is written to the SQLite WAL ring.

---

## 3. Test Verification
All 7 tests in `test_service_recovery` passed, including explicit capacity overflow and special directory handling:
```text
running 7 tests
test test_dependency_cycle_detection_in_store ... ok
test test_default_store_deep_validation ... ok
test test_negative_service_specs_and_status_invariants ... ok
test test_load_or_recover_workflow ... ok
test test_non_destructive_corruption_recovery_and_quarantine ... ok
test test_sr1_sr2_sr3_invariant_equations ... ok
test test_service_recovery_hardening ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
```
