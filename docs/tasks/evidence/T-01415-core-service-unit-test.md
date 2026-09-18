# T-01415: User Session Bootstrap - Core Service: Unit Test

## Metadata
- **Task ID:** `T-01415`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap Core Service Unit Test (`code/aiosh-rust/aiosh-core/tests/test_session_service.rs`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (5/10) — Core Service Unit Test

---

## 1. Test Suite Architecture & Coverage

Created dedicated integration and unit test suite `code/aiosh-rust/aiosh-core/tests/test_session_service.rs` providing comprehensive test coverage for criteria `CS1..CS5`:

| Test Name | Criterion | Scope & Invariants Verified | Result |
|---|---|---|---|
| `test_cs1_canonical_seeding_and_empty` | **CS1** | Canonical greeter pre-seeding (`greeter-seat0`, `lightdm`, `X11`, `Active`, `Foreground`, `leader_pid: Some(1001)`) and unseeded `empty()` initialization. | **PASS** |
| `test_cs1_session_lifecycle_state_machine` | **CS1** | Complete forward lifecycle: `Initializing` $\to$ `Authenticating` $\to$ `Active` $\to$ `Locked` $\to$ `Active` $\to$ `Terminating` $\to$ `Terminated`. | **PASS** |
| `test_cs1_negative_transitions_and_error_handling` | **CS1** | Rejection of invalid transitions (locking initializing, unlocking active), non-existent session lookups, and immutability of terminated sessions. | **PASS** |
| `test_cs2_seat_arbitration_and_foreground_uniqueness` | **CS2** | Mutual exclusion of `SessionScope::Foreground` on `seat0`; automatic background demotion upon activation; independence of separate seats (`seat1`). | **PASS** |
| `test_cs3_capacity_limits_and_user_boundaries` | **CS3** | Enforcement of 32 active sessions per user; rejection of 33rd session; recycling capacity after session termination. | **PASS** |
| `test_cs4_action_reporting_and_envelope_integrity` | **CS4** | Action report envelope correctness: fields, state transitions, RFC-3339 timestamps, Serde JSON serialization. | **PASS** |
| `test_cs5_idle_tracking_and_activity_resets` | **CS5** | Monotonic idle tracking (`idle_seconds`), activity resets (`touch_activity`), lock/unlock consistency (`locked` bool), and action-driven resets. | **PASS** |
| `test_session_query_and_atomic_persistence` | **CS1** | Multi-attribute query filtering (`username`, `seat`, `state`, `limit`), atomic write to temporary file, and round-trip deserialization via `load_from_path`. | **PASS** |

---

## 2. Test Verification Outputs

### 1. Standalone Core Service Test Suite (`cargo test -p aiosh-core --test test_session_service`)
```text
running 8 tests
test test_cs1_canonical_seeding_and_empty ... ok
test test_cs1_negative_transitions_and_error_handling ... ok
test test_cs1_session_lifecycle_state_machine ... ok
test test_cs2_seat_arbitration_and_foreground_uniqueness ... ok
test test_cs3_capacity_limits_and_user_boundaries ... ok
test test_cs4_action_reporting_and_envelope_integrity ... ok
test test_cs5_idle_tracking_and_activity_resets ... ok
test test_session_query_and_atomic_persistence ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

### 2. Master Session Suite Matrix (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

---

## 3. Acceptance Verification

- [x] New test file runs standalone and passes with 8/8 green tests.
- [x] Negative and boundary cases are explicitly asserted across invalid transitions, duplicate creations, and capacity bounds.
- [x] Integrated into master runner `tools/test_session_suites.py` as criterion `SB4`.
