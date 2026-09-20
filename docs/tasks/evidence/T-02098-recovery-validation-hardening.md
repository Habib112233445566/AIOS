# Hardening Details: T-02098

- **Task**: T-02098 (recovery & validation: Hardening)
- **Subsystem**: Capability Model Recovery & Validation
- **Controls Implemented**:
  - `validate_service_path` mandatory guard in `recover_capability_store`: rejects traversal `..`, null bytes, control characters, non-json extensions.
  - Symlink refusal in `create_backup_file`: prevents symlink exploitation during non-destructive quarantine.
  - Symlink detection in `validate_capability_store`: records error in `CapabilityValidationReport`.
  - Permission 0600 on Unix quarantine files: prevents unprivileged token harvesting.
- **Test Results**: All 9 unit tests passed without regression.
