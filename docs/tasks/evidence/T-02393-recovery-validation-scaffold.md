# Task Evidence: T-02393 - Audit Chain Extensions: Recovery & Validation Scaffold

## Goal
Create module skeleton and interfaces for the recovery & validation subsystem of Audit Chain Extensions.

## Scaffold Details
1. **Module Creation**:
   - Created `code/aiosh-rust/aiosh-core/src/audit_chain_recovery.rs`.
   - Defined types:
     - `AuditChainIssueSeverity`: `Fatal`, `Error`, `Warning`.
     - `AuditChainIssueCode`: `HashDiscontinuity`, `InvalidJson`, `SignatureMismatch`, `DanglingCausalLink`, `CausalCycleDetected`, `TimeInversion`.
     - `AuditChainValidationIssue`: per-issue diagnostic record with row ID and event hash.
     - `AuditChainValidationReport`: aggregate validation metrics, healthy counts, issue lists.
     - `AuditChainRepairAction`: actions executed during forward recovery.
     - `AuditChainRecoveryResult`: complete result containing backup snapshot path, action list, and post-repair validation.
     - `AuditChainRecoveryManager`: implements `validate(&AuditChainService)` and `recover(&mut AuditChainService, Option<&Path>)`.
2. **Error Codes**:
   - `AUDITRECV_ERR_IO`: File I/O snapshot errors.
   - `AUDITRECV_ERR_VALIDATION`: Database query or inspection errors.
   - `AUDITRECV_ERR_CORRUPT`: Unrecoverable database state.
3. **Module Registration**:
   - Exported `pub mod audit_chain_recovery;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
   - Verified clean compilation: `cargo check -p aiosh-core` passed with 0 warnings.
