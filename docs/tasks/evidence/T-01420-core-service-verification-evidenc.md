# T-01420: User Session Bootstrap - Core Service: Verification & Evidence

## Metadata
- **Task ID:** `T-01420`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap / Core Service: Verification & Evidence (`code/aiosh-rust/aiosh-core::session_service`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic Closure (10/10): User Session Bootstrap / Core Service (`T-01411..T-01420`)

---

## 1. Executive Summary

This document establishes formal verification and captures automated test evidence for the completion and closure of the **User Session Bootstrap Core Service** sub-epic (`T-01411..T-01420`), the second sub-epic in the User Session Bootstrap subsystem of Phase 1.

All 10 tasks in this sub-epic have been completed in strict sequential order under the NO-SKIP law:
1. `T-01411`: Research — Authoritative systemd-logind, PAM, POSIX process group semantics, seat arbitration, and invariants `CS1..CS5`.
2. `T-01412`: Specification — Formal `UserSessionService` coordinator, `UserSessionActionReport`, seat arbitration, and action contracts.
3. `T-01413`: Scaffold — Skeleton module `session_service.rs` and fail-loud stubs in `aiosh-core`.
4. `T-01414`: Implementation — Complete `UserSessionService` implementation (`create_session`, `apply_action`, `query_sessions`, `update_idle`, `touch_activity`, `save_to_path`, `load_from_path`) with pre-seeded canonical greeter on `seat0`.
5. `T-01415`: Unit Test — Standalone Rust unit test suite (`test_session_service.rs`) covering CS1..CS5. Added `SB4` criterion to `tools/test_session_suites.py`.
6. `T-01416`: Integration — Operator CLI surface (`aiosh session list`, `show`, `action`), MCP tools (`aios.session.list`, `get`, `action`), and smoke suite updates.
7. `T-01417`: Security Review — Threat model audit, abuse scenarios AS-01..AS-06 mitigation, PEP capability gating, and SQLite WAL audit logging.
8. `T-01418`: Hardening — 10 MiB store limit, 1 MiB spec cap, zero-leak PID+timestamp tempfile persistence, bounded rename retries, and standard error envelopes.
9. `T-01419`: Documentation — MCP `README.md` updates, copy-pasteable JSON-RPC & CLI examples, honest limitation disclosures, and evidence cross-linking.
10. `T-01420`: Verification & Evidence — Master test matrix verification, captured test outputs, `task_plan.md` milestone update, and sub-epic closure.

---

## 2. Invariant Compliance Matrix (CS1..CS5 & SB1..SB4)

| Invariant / Criterion | Description | Test Verification | Status |
|---|---|---|---|
| **CS1** | **State Monotonicity & Lifecycle**: Deterministic forward transitions (`Initializing` $\to$ `Authenticating` $\to$ `Active` $\rightleftharpoons$ `Locked` $\to$ `Terminating` $\to$ `Terminated`); invalid transitions rejected; terminated immutable. | `test_cs1_session_lifecycle_state_machine`, `test_cs1_negative_transitions_and_error_handling` | **PASS** |
| **CS2** | **Seat Arbitration**: At most one session per seat can hold `SessionScope::Foreground`. Activating a session demotes any prior foreground session on that seat to `SessionScope::Background`. | `test_cs2_seat_arbitration_and_foreground_uniqueness` | **PASS** |
| **CS3** | **Capacity Limits**: Hard limit of $\le 32$ active sessions per user; hard limit of $\le 1,024$ total sessions in store. | `test_cs3_capacity_limits_and_user_boundaries` | **PASS** |
| **CS4** | **Action Reporting**: Structured `UserSessionActionReport` returned on all actions containing `session_id`, `action`, `previous_state`, `new_state`, `success`, `error`, `timestamp`. | `test_cs4_action_reporting_and_envelope_integrity` | **PASS** |
| **CS5** | **Idle & Lock Consistency**: `idle_seconds` tracked; `touch_activity` resets idle to 0; `state == Locked` strictly matches `locked == true`; touching activity never unlocks. | `test_cs5_idle_tracking_and_activity_resets` | **PASS** |
| **Hardening** | **Zero-Leak Tempfiles & Size Caps**: 10 MiB store cap, clean unlinking of tempfiles on write/rename errors, bounded rename retries. | `test_hardening_atomic_save_and_error_cleanup`, `test_hardening_session_service_explicit_error_envelopes` | **PASS** |
| **SB1** | **Master Runner SB1**: Session data model integrity & invariants (`SB1..SB5`). | `tools/test_session_suites.py` (SB1) | **PASS** |
| **SB2** | **Master Runner SB2**: Operator CLI surface commands & options (`validate`, `list`, `show`, `action`, errors). | `tools/test_session_suites.py` (SB2) | **PASS** |
| **SB3** | **Master Runner SB3**: Autonomous agent MCP tool surface (`aios.session.validate`). | `tools/test_session_suites.py` (SB3) | **PASS** |
| **SB4** | **Master Runner SB4**: Session core service lifecycle, seat arbitration & invariants (`CS1..CS5`). | `tools/test_session_suites.py` (SB4) | **PASS** |

---

## 3. Test Suite Verification Outputs

### 3.1 Master Session Runner (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

### 3.2 Core Service Unit & Hardening Suite (`cargo test --test test_session_service`)
```text
running 10 tests
test test_cs1_canonical_seeding_and_empty ... ok
test test_cs1_negative_transitions_and_error_handling ... ok
test test_cs1_session_lifecycle_state_machine ... ok
test test_cs2_seat_arbitration_and_foreground_uniqueness ... ok
test test_cs4_action_reporting_and_envelope_integrity ... ok
test test_cs3_capacity_limits_and_user_boundaries ... ok
test test_cs5_idle_tracking_and_activity_resets ... ok
test test_hardening_session_service_explicit_error_envelopes ... ok
test test_hardening_atomic_save_and_error_cleanup ... ok
test test_session_query_and_atomic_persistence ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### 3.3 Data Model Invariant Suite (`cargo test --test test_session_data_model`)
```text
running 8 tests
test test_sb2_username_and_identity_bounds ... ok
test test_sb1_session_id_boundary_and_syntax ... ok
test test_sb3_lifecycle_state_machine_matrix ... ok
test test_sb4_environment_and_path_isolation ... ok
test test_session_query_and_filtering ... ok
test test_session_status_consistency_validation ... ok
test test_sb5_store_capacity_and_user_limits ... ok
test test_session_store_persistence_and_atomic_save ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### 3.4 CLI Surface Smoke & Hardening (`python code/aiosh-cli/tests/test_session_cli_smoke.py`)
```text
=== RUNNING USER SESSION BOOTSTRAP CLI SMOKE TESTS ===
PASS: aiosh session --help
PASS: aiosh session unknown_cmd returns 2
PASS: aiosh session validate (id, user, spec, and json)
PASS: aiosh session list, show, action
PASS: aiosh session hardening (payload limits, json parse, missing args, action envelopes)

ALL USER SESSION BOOTSTRAP CLI SMOKE TESTS PASSED!
```

---

## 4. Sub-Epic Closure & Hand-Off

With the successful verification and capture of test output across all criteria:
- **Sub-Epic Status**: **CLOSED (10/10 tasks)** (`T-01411..T-01420`).
- **Components Shipped**:
  - `code/aiosh-rust/aiosh-core/src/session_service.rs` (UserSessionService core implementation)
  - `code/aiosh-rust/aiosh-core/src/session.rs` (UserSessionStore atomic persistence & validation)
  - `code/aiosh-rust/aiosh-core/src/lib.rs` (Module export)
  - `code/aiosh-rust/aiosh-core/tests/test_session_service.rs` (10 unit & hardening tests)
  - `code/aiosh-rust/aiosh-cli/src/main.rs` (Integrated `aiosh session list`, `show`, `action`)
  - `code/aiosh-rust/aiosh-mcp/src/main.rs` (Dispatched `aios.session.list`, `get`, `action`)
  - `code/aiosh-cli/tests/test_session_cli_smoke.py` (CLI smoke test suite)
  - `tools/test_session_suites.py` (Master test runner with criteria SB1..SB4)
  - `code/aiosh-mcp/README.md` (MCP tool documentation)
- **Next Sub-Epic**: **User Session Bootstrap / CLI surface** (`T-01421..T-01430`).
- **Next Immediate Task**: **`T-01421`** (`Phase 1 — Linux Base System & Bootable Target / User Session Bootstrap / CLI surface: Research`).
