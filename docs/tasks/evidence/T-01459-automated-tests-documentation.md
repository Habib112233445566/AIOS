# T-01459: User Session Bootstrap — Automated Tests: Documentation

## Metadata
- **Task ID:** `T-01459`
- **Subsystem:** `code/aiosh-rust/aiosh-core`, `tools/test_session_suites.py`
- **Component:** User Session Bootstrap Automated Testing Subsystem
- **Status:** Complete
- **Date:** 2026-09-11

---

## 1. Documentation Deliverables

### Automated Test Suite Architecture (`test_session_automated.rs`)

The User Session Bootstrap automated testing subsystem provides end-to-end integration test coverage for the core session coordinator, state transitions, seat arbitration, resource bounds, and disk persistence:

| Suite ID | Test Name | Invariants Verified |
|---|---|---|
| **SBT1** | `test_sbt1_lifecycle_fsm_cohesion` | Validates multi-turn lifecycle state machine progression (`Initializing` -> `Authenticating` -> `Active` -> `Locked` -> `Active` -> `Terminating` -> `Terminated`) and rejection of invalid actions (e.g., locking before authentication, modifying after termination). |
| **SBT2** | `test_sbt2_seat_arbitration_and_demotion` | Enforces seat-level foreground exclusivity: activating a second session on `seat0` automatically demotes the first session to `Background`. Independent seats (`seat1`) maintain concurrent foreground sessions without cross-seat demotion. |
| **SBT3** | `test_sbt3_capacity_quotas` | Enforces `MAX_SESSIONS_PER_USER` boundary: successfully provisions 32 active sessions for a user, then deterministically rejects the 33rd session creation. |
| **SBT4** | `test_sbt4_store_persistence` | Verifies disk persistence roundtrip using `save_to_path` and `load_from_path`. Confirms identical session count, user attributes, state, and locked flags upon recovery. |
| **SBT5** | `test_sbt5_catalog_introspection` | Exercises `query_sessions` across multiple query parameters: username, session type (`AiAgent`), seat, and limit pagination. |

### Test Execution Commands

```bash
# Run standalone Rust automated test binary
cargo test --manifest-path code/aiosh-rust/Cargo.toml --test test_session_automated

# Run complete Phase 1 session criteria runner (SB1..SB6)
python tools/test_session_suites.py
```

### Known Constraints & Behavior
- Quotas are strictly enforced at runtime (`32` active sessions per user, `10,000` total store capacity).
- Sessions in `Terminated` state are permanently immutable and cannot receive subsequent administrative actions.
- Seat arbitration is strictly seat-scoped; no inter-seat interference occurs.
