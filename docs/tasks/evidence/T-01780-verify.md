# Verification Log: T-01780 Sub-Epic 8 Closure

```text
running 9 tests (aiosh-core: test_hardware_observability)
test test_hardening_metadata_sanitization ... ok
test test_ho1_class_breakdown_parity ... ok
test test_ho2_bus_breakdown_parity ... ok
test test_ho3_driver_binding_accounting ... ok
test test_ho4_driver_binding_rate ... ok
test test_ho5_policy_compliance_summary ... ok
test test_ho6_json_roundtrip ... ok
test test_observability_service_integration ... ok
test test_hardening_prohibited_devices_cap ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s

running 16 tests (aiosh-core: test_hardware_policy)
test test_hardening_case_insensitive_vendor_matching ... ok
test test_hardening_path_traversal_rejected ... ok
test test_hardening_oversized_policy_file_rejected ... ok
test test_hsec1_audit_mode_verdict ... ok
test test_hsec1_permissive_mode_verdict ... ok
test test_hsec1_precedence_prohibited_device ... ok
test test_hsec2_attribute_redaction ... ok
test test_hsec2_redaction_disabled ... ok
test test_hsec3_disallowed_bus ... ok
test test_hsec3_disallowed_class ... ok
test test_hardening_list_bound_limits ... ok
test test_hsec4_deterministic_report ... ok
test test_hsec5_json_roundtrip ... ok
test test_hsec5_validation_bounds ... ok
test test_policy_default_valid ... ok
test test_hsec5_load_save_and_missing_fallback ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

Starting Hardware Detection Observability Smoke Suite (HO1..HO6)...
PASS: test_ho1_class_breakdown_parity
PASS: test_ho2_bus_breakdown_parity
PASS: test_ho3_driver_binding_accounting
PASS: test_ho4_driver_binding_rate
PASS: test_ho5_policy_compliance_and_serialization
ALL 5 HARDWARE DETECTION OBSERVABILITY INTEGRATION TESTS PASSED.

Starting Hardware Detection Security Policy Smoke Suite (HSEC1..HSEC5)...
PASS: test_hsec1_policy_precedence
PASS: test_hsec2_attribute_redaction
PASS: test_hsec3_class_and_bus_gatekeeping
PASS: test_hsec4_deterministic_evaluation
PASS: test_hsec5_fail_safe_defaults_and_bounds
ALL 5 HARDWARE DETECTION SECURITY POLICY INTEGRATION TESTS PASSED.

Starting Hardware Detection Automated Tests Smoke Suite (AT1..AT5)...
PASS: test_at1_hermetic_isolation
PASS: test_at2_fault_injection
PASS: test_at3_deterministic_classification
PASS: test_at4_invariant_compliance
PASS: test_at5_scale_and_traversal_bound
ALL 5 HARDWARE DETECTION AUTOMATED TESTS INTEGRATION TESTS PASSED.
```
