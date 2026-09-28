# Task Evidence: T-02398 - Audit Chain Extensions: Recovery & Validation Hardening

## Goal
Harden the recovery & validation subsystem of Audit Chain Extensions against path traversal, unbound diagnostic loops, and invalid arguments.

## Hardening Implemented
1. **Path Traversal Prevention (`validate_backup_dir`)**:
   - Rejects directory traversal tokens (`..`) using component inspection.
   - Enforces `MAX_PATH_LEN = 1024` on backup paths.
   - Rejects non-printable control characters.
   - Produces structured error code `AUDITRECV_ERR_PATH_TRAVERSAL`.
2. **Diagnostic Issue Capping (`MAX_VALIDATION_ISSUES = 1000`)**:
   - Enforces a strict upper bound of 1,000 issues collected during any validation run to protect memory from uncontrolled ballooning on severely damaged databases.
3. **Structured Error Codes**:
   - Standardized error reporting: `AUDITRECV_ERR_IO`, `AUDITRECV_ERR_VALIDATION`, `AUDITRECV_ERR_CORRUPT`, `AUDITRECV_ERR_PATH_TRAVERSAL`.
4. **Automated Testing**:
   - Added unit test `test_validate_backup_dir_path_traversal` in `audit_chain_recovery.rs`.
   - Verified 5/5 unit tests and 5/5 integration tests passing.
