# T-01452: User Session Bootstrap — Automated Tests: Specification

## Metadata
- **Task ID:** `T-01452`
- **Subsystem:** `code/aiosh-rust/aiosh-core`
- **Component:** User Session Bootstrap Automated Testing Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Test Suite Contract & Architecture

The automated integration test suite `code/aiosh-rust/aiosh-core/tests/test_session_automated.rs` specifies five rigorous verification criteria (`SBT1..SBT5`) testing the cohesive operation of the User Session Bootstrap subsystem.

### Criteria Matrix:

| Criterion | Name | Assertion Contract |
|---|---|---|
| **SBT1** | FSM Lifecycle Cohesion | Exercises the entire lifecycle graph: `Initializing` $\to$ `Authenticating` $\to$ `Active` $\rightleftharpoons$ `Locked` $\to$ `Terminating` $\to$ `Terminated`. Asserts deterministic rejection of invalid backward or skipped transitions (e.g. `Terminated` $\to$ `Active`, `Initializing` $\to$ `Locked`). |
| **SBT2** | Seat Arbitration & Demotion | Verifies single-foreground seat discipline. Activating session $S_1$ on `seat0` marks it `Foreground`. Subsequently activating session $S_2$ on `seat0` marks $S_2$ as `Foreground` and automatically demotes $S_1$ to `Background`. Terminating $S_2$ releases foreground status. |
| **SBT3** | Multi-User Capacity Quotas | Verifies per-user active session saturation (boundary at 32 sessions) and rejection with `EXCEEDS_CAPACITY`. Verifies total store capacity enforcement (boundary at 1,024 sessions). |
| **SBT4** | Multi-Turn Disk Persistence | Verifies full state preservation across process invocations: `save_to_path` followed by `load_from_path` produces an identical in-memory session graph. Tests tempfile isolation and corrupt store handling. |
| **SBT5** | Multi-Parameter Query Introspection | Verifies query filtering over `UserSessionQuery` across `username`, `state`, `session_type`, `seat`, and `limit`. Asserts correct sort ordering and limit truncation. |

---

## 2. Test Harness Specification

```rust
/// Helper creating a synthetic valid session specification for testing.
pub fn create_test_session_spec(
    session_id: &str,
    username: &str,
    uid: u32,
    gid: u32,
    session_type: SessionType,
    session_class: SessionClass,
    seat: &str,
) -> UserSessionSpec;
```

---

## 3. Master Test Runner Integration (`SB6`)

The master test runner `tools/test_session_suites.py` will incorporate:
- `test_sb6_automated_integration`: Runs `cargo test --test test_session_automated`.
- Full suite pass condition: `(SB1..SB6)`.
