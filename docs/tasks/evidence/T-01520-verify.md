# T-01520: Filesystem Layout - Core Service: Verification & Evidence

## Metadata
- **Task ID:** `T-01520`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout Core Service (`code/aiosh-rust/aiosh-core::fs_layout_service`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (10/10) — Core Service Verification & Evidence (MILESTONE COMPLETE)
- **Dependencies:** `T-01519` (Core Service Documentation)
- **Next Task:** `T-01521` (Filesystem Layout / CLI surface: Research)

---

## 1. Executive Summary & Verification Matrix

Task `T-01520` executes the full verification battery for the **Filesystem Layout Core Service** sub-epic (Tasks `T-01511..T-01520`). The suite confirms mathematical consistency, invariant enforcement (`CS1..CS5`), target disk capacity feasibility checks, differential safety analysis, atomic file persistence, CLI subcommand execution, MCP tool invocation, and full baseline smoke suite integrity with zero regressions.

### 1.1 Test Battery Execution

```
======================================================================
1. aiosh-core::fs_layout_service Unit Test Suite (11 tests)
======================================================================
$ cargo test -p aiosh-core --test test_fs_layout_service
running 11 tests
test test_fs_layout_service_diff_layouts ... ok
test test_fs_layout_service_fstab_export_and_import ... ok
test test_fs_layout_service_hardening_mount_caps_and_path_hygiene ... ok
test test_fs_layout_service_atomic_persistence ... ok
test test_fs_layout_service_negative_registration_and_validation ... ok
test test_fs_layout_service_negative_store_and_diff_operations ... ok
test test_fs_layout_service_probe_boundary_and_slack_warnings ... ok
test test_fs_layout_service_probe_target ... ok
test test_fs_layout_service_scaffold_initialization ... ok
test test_fs_layout_service_store_crud ... ok
test test_fs_layout_service_negative_fstab_and_persistence ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

======================================================================
2. aiosh-core::fs_layout Data Model Test Suite (19 tests)
======================================================================
$ cargo test -p aiosh-core --test test_fs_layout_data_model
running 19 tests
test test_directory_spec_validation ... ok
test test_fl2_path_control_chars_and_oversized ... ok
test test_fl1_single_root_mount_enforcement ... ok
test test_fl1_root_pass_number_validation ... ok
test test_fl2_path_hygiene ... ok
test test_fl3_mount_order_hierarchy ... ok
test test_fl3_duplicate_mount_paths ... ok
test test_fl4_cis_security_options ... ok
test test_fl4_dev_shm_cis_options ... ok
test test_fl5_partition_constraints ... ok
test test_fl5_esp_mount_filesystem_mismatch ... ok
test test_fs_type_roundtrip_and_custom ... ok
test test_fl5_total_partition_budget_exceeded ... ok
test test_fstab_malformed_lines ... ok
test test_minimal_container_layout_validity ... ok
test test_fstab_serialization_and_parsing ... ok
test test_partition_type_gpt_guid_mapping ... ok
test test_standard_uefi_layout_validity ... ok
test test_layout_json_roundtrip ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

======================================================================
3. aiosh-cli Integration Test (aiosh layout flow)
======================================================================
$ cargo test -p aiosh-cli --bin aiosh test_cmd_fs_layout_flow
running 1 test
test task_cli_tests::test_cmd_fs_layout_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 0.36s

======================================================================
4. aiosh-mcp Integration Test (aios.fs_layout.* tools)
======================================================================
$ cargo test -p aiosh-mcp --bin aiosh-mcp test_mcp_fs_layout_tools
running 1 test
test tests::test_mcp_fs_layout_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.08s

======================================================================
5. Baseline Master Smoke Suites
======================================================================
$ python tools/test_session_suites.py
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP in-tree unit test suite
[+] SB3 session MCP tool surface JSON-RPC smoke test (aios.session.*)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)
[+] SB5 session configuration resolution, invariants & precedence (SC1..SC7)
[+] SB6 session automated integration test suite (SBT1..SBT5)
[+] SB7 session security policy enforcement & invariants (SSP1..SSP7)
[+] SB8 session observability & telemetry metrics (SSO1..SSO6)
[+] SB9 session documentation architecture & operational guide (D1..D6)

PASS: session_suites criteria (SB1..SB9)

$ python tools/test_session_doc.py
[+] D1 doc existence and size bounds (26647 bytes)
[+] D2 all 9 required sections present
[+] D3 zero forbidden placeholders/markers
[+] D4 policy invariants, CLI commands, and MCP tools coverage complete
[+] D5 negative rejection assertions verified
[+] D6 zero volatile snapshot counts (C6 compliant)

PASS: test_session_doc criteria (D1..D6)

$ python code/aiosh-mcp/tests/test_session_mcp_smoke.py
=== RUNNING USER SESSION BOOTSTRAP MCP SMOKE TESTS ===
PASS: tools/list contains all 6 aios.session.* tools
PASS: aios.session.validate (valid, invalid, boundary, missing)
PASS: aios.session.list & aios.session.get (filtering, inspection, error modes)
PASS: aios.session.action (PEP enforcement, lock, unlock, unknown action, missing params)
PASS: aios.session.create & persistence (PEP enforcement, create, get, duplicate rejection, invalid spec)
PASS: Cross-surface CLI <-> MCP parity & state sharing
PASS: aios.session.check validation and quarantine recovery
PASS: MCP session hardening (payload limits, query bounds, ID injection, store path sanitization)

ALL USER SESSION BOOTSTRAP MCP SMOKE TESTS PASSED!
```

---

## 2. Milestone Conclusion & State Advances

- **Sub-Epic Status**: `Filesystem Layout / core service` (10/10 tasks, `T-01511..T-01520`) **COMPLETE**.
- **State Updates**:
  - `task_plan.md` updated with milestone achievements and architectural summaries.
  - `progress.md` updated with chronological progress record.
- **Next Task**: `T-01521` (`Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / CLI surface: Research`).
