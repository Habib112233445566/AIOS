# Task Evidence: T-02213 (Grant Lifecycle / core service: Scaffold)

## 1. Scope & Execution
Created the module skeleton and typed interfaces for `PepGrantService` (`code/aiosh-rust/aiosh-core/src/pep_grant_service.rs`) and registered it in `code/aiosh-rust/aiosh-core/src/lib.rs`.

### Scaffolding Highlights:
- Defined core constants: `MAX_GRANTS_IN_SERVICE = 5000`, `MAX_GRANT_SERVICE_STORE_SIZE = 10 MiB`.
- Defined error domain strings: `GSVC_ERR_CAPACITY`, `GSVC_ERR_NOT_FOUND`, `GSVC_ERR_INVALID_TRANSITION`, `GSVC_ERR_ATTENUATION`, `GSVC_ERR_QUOTA_EXCEEDED`, `GSVC_ERR_EXPIRED`, `GSVC_ERR_NOT_YET_VALID`, `GSVC_ERR_VALIDATION`, `GSVC_ERR_IO`.
- Defined helper function `validate_grant_service_path(path: &Path) -> Result<(), String>`.
- Defined `PepGrantService` struct with primary registry (`grants`) and secondary indexing structures (`by_subject`, `by_parent`, `by_state`), plus optional `storage_path`.
- Defined typed function signatures for all lifecycle operations (`issue_grant`, `get_grant`, `list_grants`, `list_grants_for_subject`, `list_grants_by_state`, `transition_grant`, `attenuate_grant`, `evaluate_grant`, `record_grant_usage`, `revoke_grant`, `sweep_expired`, `save_to_path`, `load_from_path`, `sync`).
- Created initial test stub in `code/aiosh-rust/aiosh-core/tests/test_pep_grant_service.rs` verifying service instantiation and initial empty state.

---

## 2. Verification Results
```
> cargo check -p aiosh-core
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 49.05s
[ZERO COMPILER WARNINGS]

> cargo test -p aiosh-core --test test_pep_grant_service
running 1 test
test test_pep_grant_service_scaffold_creation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

---

## 3. Acceptance Confirmation
- [x] Project builds and passes type checking with zero errors and zero warnings.
- [x] New interfaces exist and are referenced by `tests/test_pep_grant_service.rs`.
