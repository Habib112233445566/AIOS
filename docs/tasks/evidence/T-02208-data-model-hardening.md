# Task Evidence: T-02208 (Grant Lifecycle / data model: Hardening)

## 1. Scope & Hardening Invariants
Hardened the Grant Lifecycle Data Model (`pep_grant.rs`) against failure modes, resource exhaustion, and denial of service:
1. **Size Caps & Memory Limits**:
   - `MAX_METADATA_ENTRIES = 64`: Bounds the total count of metadata properties.
   - `MAX_METADATA_KEY_LEN = 64`: Bounds individual metadata key length.
   - `MAX_METADATA_VALUE_LEN = 512`: Bounds individual metadata value length.
   - `MAX_DELEGATION_DEPTH_LIMIT = 8`: Bounds delegation recursion depth.
   - `MAX_GRANTS_IN_STORE = 5000`: Hard cap on in-memory store capacity.
   - `MAX_GRANT_STORE_SIZE = 10 MiB`: File size limit verified prior to loading/deserializing store files.
2. **Persistence Hardening & Atomic Cleanup**:
   - Collision-resistant temporary file naming incorporating process ID and nanosecond timestamps (`tmp.<pid>.<nonce>`).
   - Clean removal of temporary files on rename failure (`let _ = std::fs::remove_file(&tmp_path)`).
   - Directory validation: `load_from_path` explicitly rejects directory paths (`meta.is_dir()`) returning explicit validation errors.
3. **Explicit Error Envelopes**:
   - All failure modes return standardized, structured error codes (`PEPGRANT_ERR_VALIDATION`, `PEPGRANT_ERR_ATTENUATION`, `PEPGRANT_ERR_QUOTA_EXCEEDED`, `PEPGRANT_ERR_EXPIRED`, `PEPGRANT_ERR_INVALID_TRANSITION`, `PEPGRANT_ERR_INVALID_ID`, `PEPGRANT_ERR_NOT_YET_VALID`).
   - Standard result envelopes maintained across CLI (`{"ok": false, "error": "..."}`) and MCP (`CallToolResult` with `isError: true` and SQLite audit row).

---

## 2. Verification Results
Automated unit tests in `code/aiosh-rust/aiosh-core/tests/test_pep_grant.rs`:
```
running 10 tests
test test_pep_grant_action_validation ... ok
test test_pep_grant_attenuation ... ok
test test_pep_grant_fsm_transitions ... ok
test test_pep_grant_hardening_bounds ... ok
test test_pep_grant_invalid_identifier_and_scope ... ok
test test_pep_grant_store_hardening_file_limits ... ok
test test_pep_grant_store_operations_and_cascade_revocation ... ok
test test_pep_grant_store_atomic_persistence ... ok
test test_pep_grant_temporal_and_quota_evaluation ... ok
test test_pep_grant_valid_creation_and_validation ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

---

## 3. Acceptance Confirmation
- [x] Failure modes produce explicit, auditable errors.
- [x] No temp/connection leaks on the error path.
- [x] Memory, metadata, file size, and delegation depth limits enforced.
