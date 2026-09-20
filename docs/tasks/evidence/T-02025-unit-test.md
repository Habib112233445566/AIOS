# Task Evidence: T-02025 - Capability Model / CLI surface: Unit Test (Phase 2, Sub-Epic 3)

## 1. Overview
- **Task ID**: `T-02025`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 3 (CLI surface)
- **Goal**: Implement and verify comprehensive unit tests for `aiosh capability` CLI commands in `code/aiosh-rust/aiosh-cli`.

---

## 2. Unit Tests Added
In `code/aiosh-rust/aiosh-cli/src/main.rs` module `capability_cli_tests`:
1. `test_capability_cli_help_and_unknown`:
   - Validates help flags (`--help`, `-h`, empty) return exit code 0.
   - Validates unknown subcommands return exit code 2 and structured error payload.
2. `test_capability_cli_path_hygiene`:
   - Enforces path validation on `--store`: length $\le 1024$, `.json` extension requirement, no control characters, and no `..` traversal.
3. `test_capability_cli_issue_show_attenuate_revoke_flow`:
   - Tests complete lifecycle:
     - `list` on empty store.
     - `issue` with unauthorized issuer (rejected with exit code 1).
     - `issue` with authorized issuer `kernel` (succeeds with exit code 0).
     - `show` existing capability vs non-existent capability.
     - `attenuate` child capability with subset of rights.
     - `check` access for granted right (returns 0) vs ungranted right (returns 1).
     - `revoke` parent capability and verify cascade to child capability.
     - `check` access after revocation (returns 1).
     - `prune` expired capabilities.

---

## 3. Test Execution
- Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli --bin aiosh -- capability_cli_tests`
- Result: 3 passed; 0 failed; 0 ignored.
