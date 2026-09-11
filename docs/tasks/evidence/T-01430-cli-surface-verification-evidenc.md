# T-01430: User Session Bootstrap - CLI Surface: Verification & Evidence

## Metadata
- **Task ID:** `T-01430`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap / CLI Surface: Verification & Evidence (`code/aiosh-rust/aiosh-cli::cmd_session`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic Closure (10/10): User Session Bootstrap / CLI Surface (`T-01421..T-01430`)

---

## 1. Executive Summary

This document establishes formal verification and captures automated test evidence for the completion and closure of the **User Session Bootstrap CLI Surface** sub-epic (`T-01421..T-01430`), the third sub-epic in the User Session Bootstrap subsystem of Phase 1.

All 10 tasks in this sub-epic have been completed in strict sequential order under the NO-SKIP law:
1. `T-01421`: Research — Prior art analysis (`loginctl(1)`, PAM, seat arbitration, POSIX exit codes, and CLI taxonomy).
2. `T-01422`: Specification — Formal specification of `cmd_session`, subcommands (`validate`, `list`, `show`, `status`, `action`, `create`, shortcuts), result envelopes, and audit contracts.
3. `T-01423`: Scaffold — Subcommand routing skeleton and fail-loud stubs in `code/aiosh-rust/aiosh-cli/src/main.rs`.
4. `T-01424`: Implementation — Full implementation of `create`, `status`, and shortcut subcommands with 1 MiB spec payload bounds and atomic persistence.
5. `T-01425`: Unit Test — In-tree unit test `task_cli_tests::test_cmd_session_flow` in `main.rs` covering all command variants and error paths.
6. `T-01426`: Integration — Cross-substrate CLI and MCP interoperability, option forwarding, and SQLite WAL audit logging.
7. `T-01427`: Security Review — Threat modeling covering abuse scenarios AS-01..AS-06, path traversal rejection, payload bounds, and audit non-repudiation.
8. `T-01428`: Hardening — Enforced 1 MiB payload cap, store path bounds ($\le 1024$ bytes and control char rejection), `--limit` positive integer bounds $[1 \dots 10,000]$, and zero-leak file safety.
9. `T-01429`: Documentation — Updated `code/aiosh-cli/README.md` with complete command index, copy-pasteable examples, parameter references, and honest constraints disclosures.
10. `T-01430`: Verification & Evidence — Captured complete green test matrix across in-tree unit tests, CLI smoke suites, and master subsystem runners; updated `task_plan.md` milestone.

---

## 2. Verification Test Suite Matrix & Outputs

### 2.1 CLI Smoke Suite (`python code/aiosh-cli/tests/test_session_cli_smoke.py`)
```text
=== RUNNING USER SESSION BOOTSTRAP CLI SMOKE TESTS ===
PASS: aiosh session --help
PASS: aiosh session unknown_cmd returns 2
PASS: aiosh session validate (id, user, spec, and json)
PASS: aiosh session list, show, action
PASS: aiosh session commands (status, shortcuts, create)
PASS: aiosh session hardening (payload limits, json parse, missing args, action envelopes, store & limit checks)

ALL USER SESSION BOOTSTRAP CLI SMOKE TESTS PASSED!
```

### 2.2 Master Session Subsystem Suite (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

### 2.3 In-Tree Rust CLI Unit Test (`cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli --bin aiosh test_cmd_session_flow`)
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.54s
     Running unittests src\main.rs (code\aiosh-rust\target\debug\deps\aiosh-8814d271a6af31ad.exe)

running 1 test
test task_cli_tests::test_cmd_session_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.30s
```

### 2.4 Core Service Unit & Hardening Suite (`cargo test --manifest-path code/aiosh-rust/Cargo.toml --test test_session_service`)
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.43s
     Running tests\test_session_service.rs (code\aiosh-rust\target\debug\deps\test_session_service-1c3522de3de4e61a.exe)

running 10 tests
test test_cs1_canonical_seeding_and_empty ... ok
test test_cs1_negative_transitions_and_error_handling ... ok
test test_cs1_session_lifecycle_state_machine ... ok
test test_cs2_seat_arbitration_and_foreground_uniqueness ... ok
test test_cs4_action_reporting_and_envelope_integrity ... ok
test test_cs5_idle_tracking_and_activity_resets ... ok
test test_cs3_capacity_limits_and_user_boundaries ... ok
test test_hardening_session_service_explicit_error_envelopes ... ok
test test_hardening_atomic_save_and_error_cleanup ... ok
test test_session_query_and_atomic_persistence ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### 2.5 Data Model Invariant Suite (`cargo test --manifest-path code/aiosh-rust/Cargo.toml --test test_session_data_model`)
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.74s
     Running tests\test_session_data_model.rs (code\aiosh-rust\target\debug\deps\test_session_data_model-677216c69b7794e8.exe)

running 8 tests
test test_sb2_username_and_identity_bounds ... ok
test test_sb1_session_id_boundary_and_syntax ... ok
test test_sb3_lifecycle_state_machine_matrix ... ok
test test_sb4_environment_and_path_isolation ... ok
test test_session_status_consistency_validation ... ok
test test_session_query_and_filtering ... ok
test test_sb5_store_capacity_and_user_limits ... ok
test test_session_store_persistence_and_atomic_save ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### 2.6 Full Workspace Core Library Test Matrix (`cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --lib`)
```text
test result: ok. 351 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.18s
```

---

## 3. Subsystem Invariant & Compliance Status

| Requirement / Invariant | Enforcement Layer | Status |
|---|---|---|
| **Sizing Caps (1 MiB Payload)** | `cmd_session` pre-read ceiling check | **PASS** |
| **Path Bounds ($\le 1024$, No Control Chars)** | `cmd_session` store parameter validation | **PASS** |
| **Query Limits ($1 \le n \le 10,000$)** | `cmd_session` integer parsing & bounds check | **PASS** |
| **Standardized Error Envelopes** | Deterministic JSON envelopes across all failure codes | **PASS** |
| **Zero-Leak Persistence** | Atomic PID+nanosecond tempfiles with cleanup on error | **PASS** |
| **Seat Mutual Exclusion (CS2)** | `UserSessionService::apply_action` foreground demotion | **PASS** |
| **Monotonic Terminal Sink (CS1)** | Immutable `Terminated` state | **PASS** |
| **Audit Non-Repudiation (ADR-0035 §F-2)** | SQLite WAL append-only hash-chained logging | **PASS** |

---

## 4. Milestone & Ledger Transition

- **Sub-Epic Closed**: `Phase 1 — Linux Base System & Bootable Target / User Session Bootstrap / CLI surface` (10/10 tasks complete: `T-01421..T-01430`).
- **Milestone Updated**: `task_plan.md` updated with closure summary and verification evidence links.
- **Next Sub-Epic**: `Phase 1 — Linux Base System & Bootable Target / User Session Bootstrap / MCP API surface` starting with task **`T-01431`** (`MCP API surface: Research`).

---

## 5. Acceptance Verification
- [x] Full relevant test suite green with captured output.
- [x] Milestone updated in `task_plan.md`.
- [x] State files updated; ready to advance next task pointer to `1431`.
