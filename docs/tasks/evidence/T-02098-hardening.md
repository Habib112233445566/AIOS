# Hardening Evidence: T-02098 (recovery & validation: Hardening)

- **Target File**: `code/aiosh-rust/aiosh-core/src/capability_recovery.rs`
- **Hardening Applied**:
  1. **Path Traversal & Format Defense**:
     - Added mandatory `validate_service_path(store_path)` check at the entry of `recover_capability_store()` and inside `validate_capability_store()`.
     - Rejects any paths with `..`, control characters, non-`.json` extensions, or length $> 1024$.
  2. **Symlink Redirection Defense**:
     - Added `fs::symlink_metadata()` checks in `create_backup_file()` to immediately refuse copying if the target is a symlink.
     - Added symlink checks in `validate_capability_store()` reporting an error if the store path is a symlink.
  3. **Permission Hardening**:
     - Enforced permission `0600` on quarantine backup files across Unix platforms.
- **Verification**:
  - `cargo test --test test_capability_recovery`: 9/9 passed in 0.12s.
- **Status**: Completed.
