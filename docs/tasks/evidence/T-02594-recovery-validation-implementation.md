# T-02594: Privilege Escalation Prevention Recovery & Validation Implementation

- **Task**: `T-02594`
- **Sub-Epic**: Privilege Escalation Prevention / recovery & validation
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Implementation Overview
1. **Recovery & Invariant Validation Manager**:
   - Implemented `PrivilegeRecoveryManager` in `code/aiosh-rust/aiosh-core/src/privilege_recovery.rs`.
   - Comprehensive diagnostic validation in `validate_store_file` and `validate_raw_json`:
     - Checks JSON structure, format bounds (1 MiB ceiling).
     - Traversal and control character checks in paths and actor identifiers.
     - Detects illegal `SystemKernel` assignments (`PrivilegeIssueCode::IllegalKernelTier`).
     - Detects grant inconsistencies where active elevation lacks grant tokens.
   - Non-destructive repair in `repair_store_file`:
     - Point-in-time timestamped backup creation (`<store>.bak.<ts>`).
     - Unparseable JSON quarantine (`<store>.quarantine.<ts>`).
     - Auto-demotes illegal kernel contexts to `User`.
     - Resets inconsistent active elevation flags.
     - Atomically writes safe clean store file.
2. **CLI Surface**:
   - Implemented `aiosh privilege validate` and `aiosh privilege repair` with dual text and `--json` outputs.
   - Integrated full audit logging via `classify_and_emit`.
3. **MCP Tool Surface**:
   - Registered and implemented `aios.privilege.validate` and `aios.privilege.repair` tools via `dispatch::recorded_call`.
4. **Verification**:
   - Workspace compiles with 0 errors and 0 warnings.
