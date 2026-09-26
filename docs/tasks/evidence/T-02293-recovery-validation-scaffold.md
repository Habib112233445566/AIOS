# T-02293: Grant Lifecycle Recovery & Validation Scaffold

## Overview
This task creates the scaffold for the PEP Grant Store Recovery and Validation subsystem (`code/aiosh-rust/aiosh-core/src/pep_grant_recovery.rs`), exporting the primary types, error constants, and recovery manager into the core library crate.

## Scaffold Elements Created

### 1. Source Module
- **Path**: `code/aiosh-rust/aiosh-core/src/pep_grant_recovery.rs`
- **Module Exports**: Added to `code/aiosh-rust/aiosh-core/src/lib.rs`:
  - `pub mod pep_grant_recovery;`
  - Re-exports of `PepGrantIssueSeverity`, `PepGrantIssueCode`, `PepGrantValidationIssue`, `PepGrantValidationReport`, `PepGrantRepairAction`, `PepGrantRecoveryResult`, `PepGrantRecoveryManager`.

### 2. Error Constants
- `PEPGRANTRECV_ERR_IO`
- `PEPGRANTRECV_ERR_PARSE`
- `PEPGRANTRECV_ERR_PATH_TRAVERSAL`
- `PEPGRANTRECV_ERR_FILE_SIZE`
- `PEPGRANTRECV_ERR_VALIDATION`
- `PEPGRANTRECV_ERR_CORRUPT`

### 3. Core Trait & Interface Contracts
- `PepGrantRecoveryManager::validate_grants(grants: &[PepGrant]) -> PepGrantValidationReport`
- `PepGrantRecoveryManager::validate_store_file<P: AsRef<Path>>(path: P) -> Result<PepGrantValidationReport, String>`
- `PepGrantRecoveryManager::recover_store_file<P: AsRef<Path>>(path: P, dry_run: bool) -> Result<PepGrantRecoveryResult, String>`
