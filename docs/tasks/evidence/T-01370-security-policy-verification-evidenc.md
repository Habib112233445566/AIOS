# T-01370: Init & Service Supervision / Security Policy - Verification & Evidence

## Metadata
- **Task ID:** `T-01370`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Security Policy
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Executive Summary
This document provides final verification and test evidence for the completion of the Init & Service Supervision Security Policy milestone (`T-01361..T-01370`). All invariants `SP1..SP6` have been implemented, tested, hardened, and integrated into the CLI (`aiosh service policy`), MCP (`aios.service.policy`), and the master standalone suite (`tools/test_service_suites.py`).

---

## 2. Invariant Compliance Matrix (SP1..SP6)

| Invariant | Description | Verification Test | Status |
|---|---|---|---|
| **SP1** | Policy Configuration Bounds | `test_sp1_policy_configuration_bounds_and_defaults` | **PASS** |
| **SP2** | Prohibited Service Rejection (`telnet`, `rsh`, `tftp`, etc.) | `test_sp2_prohibited_service_blocking` | **PASS** |
| **SP3** | Executable Path & Traversal Hygiene (`/tmp`, `..`) | `test_sp3_executable_path_and_working_dir_hygiene` | **PASS** |
| **SP4** | User Privilege & Root Restriction (`disallow_root`, `require_service_user`) | `test_sp4_user_privilege_and_root_hygiene` | **PASS** |
| **SP5** | Environment Variable & Timeout Sanitization (`LD_PRELOAD`, `IFS`) | `test_sp5_environment_and_parameter_sanitization` | **PASS** |
| **SP6** | Tri-State Policy Modes (`Enforcing`, `Audit`, `Permissive`) & Audit Logging | `test_sp6_policy_modes_store_evaluation_and_file_roundtrip` | **PASS** |
| **Hardening** | Boundary Defenses, Path Lengths & Fail-Closed Semantics | `test_sp7_hardening_and_boundary_checks` | **PASS** |

---

## 3. Test Suite Verification Outputs

### 1. Standalone Service Subsystem Suite (`tools/test_service_suites.py`)
```text
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate, list, show/status, action, order)
[+] SS3 service MCP tool surface (validate, list, get, action, order)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)
[+] SS5 service configuration subsystem invariants, precedence & sizing (SC1..SC7)
[+] SS6 service automated integration tests (ST1..ST5)
[+] SS7 service security policy invariants & evaluation (SP1..SP6)

PASS: service_suites criteria (SS1..SS7)
```

### 2. Integration Test Suite (`tests/test_service_policy.rs`)
```text
running 7 tests
test test_sp2_prohibited_service_blocking ... ok
test test_sp4_user_privilege_and_root_hygiene ... ok
test test_sp3_executable_path_and_working_dir_hygiene ... ok
test test_sp1_policy_configuration_bounds_and_defaults ... ok
test test_sp7_hardening_and_boundary_checks ... ok
test test_sp5_environment_and_parameter_sanitization ... ok
test test_sp6_policy_modes_store_evaluation_and_file_roundtrip ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

### 3. MCP Tool Suite (`aiosh-mcp`)
```text
running 10 tests
test tests::test_mcp_distro_tools ... ok
test tests::test_mcp_handoff_tools ... ok
test tests::test_mcp_image_tools ... ok
test tests::test_mcp_package_tools ... ok
test tests::test_mcp_service_tools ... ok
test tests::test_mcp_triage_tools ... ok
test tests::test_toolchain_tools_in_manifest ... ok
test tests::test_mcp_doc_tools_execution ... ok
test tests::test_mcp_repo_health_execution ... ok
test tests::test_mcp_secrets_tools_execution ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.99s
```

### 4. CLI Surface Test Suite (`aiosh-cli`)
```text
running 22 tests
test task_cli_tests::double_dash_allows_dash_leading_values ... ok
test task_cli_tests::evidence_item_cap_enforced_by_validate ... ok
test task_cli_tests::extra_operand_rejected_for_read_only_actions ... ok
test task_cli_tests::id_must_be_decimal_gte_one ... ok
test task_cli_tests::parses_done_with_note_and_repeatable_evidence ... ok
test task_cli_tests::parses_status_without_operand ... ok
test task_cli_tests::rejects_dash_leading_option_value ... ok
test task_cli_tests::rejects_empty_note ... ok
test task_cli_tests::rejects_missing_note ... ok
test task_cli_tests::rejects_missing_value_at_end ... ok
test task_cli_tests::rejects_unknown_option_token ... ok
test task_cli_tests::rejects_oversized_text_at_validate ... ok
test task_cli_tests::test_cmd_handoff_flow ... ok
test task_cli_tests::test_cmd_doc_show_check_and_search ... ok
test task_cli_tests::test_cmd_distro_flow ... ok
test task_cli_tests::test_cmd_image_flow ... ok
test task_cli_tests::test_cmd_package_flow ... ok
test task_cli_tests::test_cmd_triage_flow ... ok
test task_cli_tests::usage_text_lists_contract ... ok
test task_cli_tests::test_cmd_service_flow ... ok
test task_cli_tests::test_cmd_repo_health_and_check ... ok
test task_cli_tests::test_cmd_secrets_scan_and_check ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.85s
```

### 5. Documentation Invariants (`tools/check_task_docs.py`)
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```

---

## 4. Conclusion & Milestone Advancement
All tasks `T-01361` through `T-01370` have been completed with zero regressions. Milestone **Init & Service Supervision / Security Policy** is officially **CLOSED**.
Advancing to next task `T-01371`.
