# T-01405: User Session Bootstrap - Data Model: Unit Test

## Metadata
- **Task ID:** `T-01405`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Data Model Unit Tests (`code/aiosh-rust/aiosh-core::session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (5/10) — Data Model Unit Test

---

## 1. Test Suite Summary
Created dedicated standalone integration and unit test suite `code/aiosh-rust/aiosh-core/tests/test_session_data_model.rs` asserting all invariants `SB1..SB5` of the User Session Bootstrap data model.

### Test Matrix & Invariant Coverage:
1. **`test_sb1_session_id_boundary_and_syntax`**:
   - Asserts valid standard IDs (`sess-01`, `c1`, `2`, `agent_session.42`).
   - Asserts length boundaries (min 1 char, max 64 chars).
   - Asserts negative rejection: empty strings, oversized IDs (65 chars), leading symbols (`-sess`, `.sess`, `_sess`), whitespace, path separators (`/`, `\`), path traversal (`..`, `../evil`, `sess..01`), shell metacharacters (`;`, `&`, `|`, `>`, `<`, `$`), and null bytes (`\0`).

2. **`test_sb2_username_and_identity_bounds`**:
   - Asserts valid POSIX user identifiers (`kali`, `root`, `_apt`, `aios-agent`, `daemon`).
   - Asserts length boundaries (min 1 char, max 32 chars).
   - Asserts negative rejection: empty strings, oversized usernames (33 chars), uppercase names (`Kali`, `ROOT`), leading digits (`1user`), whitespace, slashes, metacharacters, and null bytes.

3. **`test_sb3_lifecycle_state_machine_matrix`**:
   - Asserts complete happy path state transitions: `Initializing` $\to$ `Authenticating` $\to$ `Active` $\to$ `Locked` $\to$ `Active` $\to$ `Terminating` $\to$ `Terminated`.
   - Asserts early and direct termination transitions (`Initializing` $\to$ `Terminated`, `Authenticating` $\to$ `Terminated`, `Locked` $\to$ `Terminating`).
   - Asserts negative transitions: cannot activate or modify a `Terminated` session, cannot skip authentication (`Initializing` $\to$ `Active`), and cannot lock an `Initializing` session.

4. **`test_sb4_environment_and_path_isolation`**:
   - Asserts boundary behavior: 256 environment keys accepted.
   - Asserts rejection of oversized environment maps (>256 keys).
   - Asserts rejection of lowercase or invalid environment key names.
   - Asserts rejection of null bytes in environment values.
   - Asserts rejection of relative paths or path traversal (`..`) in `XDG_RUNTIME_DIR`.

5. **`test_sb5_store_capacity_and_user_limits`**:
   - Asserts duplicate session ID rejection.
   - Asserts per-user session capacity saturation (user `kali` can open up to 32 active sessions, 33rd is rejected).
   - Asserts multi-tenant capacity (another user such as `root` can open sessions even when another user has reached limit).

6. **`test_session_status_consistency_validation`**:
   - Rejects contradictory states: `Locked` state with `locked == false`, `Active` state with `locked == true`, `Terminated` state with `Foreground` scope.

7. **`test_session_store_persistence_and_atomic_save`**:
   - Asserts serialization and deserialization symmetry.
   - Asserts atomic file write via temporary file replacement.
   - Asserts enforcement of the 10 MiB payload size limit on deserialization.

8. **`test_session_query_and_filtering`**:
   - Validates multi-attribute search across username, state, session type, seat, and count limits.

---

## 2. Test Execution Output (`cargo test --test test_session_data_model`)
```text
running 8 tests
test test_sb1_session_id_boundary_and_syntax ... ok
test test_sb2_username_and_identity_bounds ... ok
test test_sb3_lifecycle_state_machine_matrix ... ok
test test_sb4_environment_and_path_isolation ... ok
test test_sb5_store_capacity_and_user_limits ... ok
test test_session_query_and_filtering ... ok
test test_session_status_consistency_validation ... ok
test test_session_store_persistence_and_atomic_save ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

---

## 3. Acceptance Verification
- [x] New standalone test file `test_session_data_model.rs` runs in isolation and passes 100%.
- [x] Negative and boundary cases thoroughly asserted across all formal invariants (`SB1..SB5`).
