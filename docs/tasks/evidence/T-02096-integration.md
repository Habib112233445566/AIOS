# Integration Evidence: T-02096 (recovery & validation: Integration)

- **Target Files**:
  - `code/aiosh-rust/aiosh-mcp/src/main.rs`
  - `code/aiosh-mcp/tests/test_capability_recovery_smoke.py`
- **Exposed Endpoints**:
  - `aios.capability.recover`: Validates store, automatically quarantines damaged/corrupted files to `.bak.<timestamp>`, and creates a clean default store.
  - `aios.capability.validate`: Deep validation of store integrity, checking invariants `CAPREC1..CAPREC4` without mutating files.
- **Verification**:
  - `test_capability_recovery_smoke.py`:
    - Registration check in `tools/list`: **PASSED**
    - Non-existent store auto-initialization (`CreatedDefaultFresh`): **PASSED**
    - Clean store validation (`healthy: true`, 0 errors): **PASSED**
    - Corrupted store quarantine & recovery (`RecoveredFromBackup`): **PASSED**
- **Status**: Completed and fully operational.
