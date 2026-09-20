# T-01765: Hardware Detection — Security Policy Unit Test

## Metadata
- **Task ID**: `T-01765`
- **Sub-Epic**: Sub-Epic 7: Hardware Detection Security Policy
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Test Coverage Overview
Implemented 12 unit tests in `code/aiosh-rust/aiosh-core/tests/test_hardware_policy.rs` covering all invariants HSEC1..HSEC5:
- `test_policy_default_valid`: Asserts default policy passes validation and has safe defaults.
- `test_hsec1_precedence_prohibited_device`: Asserts prohibited device IDs generate fatal violations, yield "deny", and are filtered in Enforcing mode.
- `test_hsec1_audit_mode_verdict`: Asserts Audit mode generates "audit" verdict without stripping devices.
- `test_hsec1_permissive_mode_verdict`: Asserts Permissive mode yields "allow" verdict.
- `test_hsec2_attribute_redaction`: Asserts sensitive attributes (`address`, `serial_number`, `uuid`) are masked to `"<REDACTED>"`.
- `test_hsec2_redaction_disabled`: Asserts attributes remain untouched when redaction is disabled.
- `test_hsec3_disallowed_class`: Asserts disallowed device class generates fatal violation.
- `test_hsec3_disallowed_bus`: Asserts disallowed device bus generates fatal violation.
- `test_hsec4_deterministic_report`: Asserts deterministic evaluation reports.
- `test_hsec5_validation_bounds`: Asserts validation of device counts, prohibited ID strings, and vendor hex format.
- `test_hsec5_json_roundtrip`: Asserts lossless JSON serialization.
- `test_hsec5_load_save_and_missing_fallback`: Asserts atomic save and missing file fallback.
