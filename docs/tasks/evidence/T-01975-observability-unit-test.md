# Task Evidence: T-01975 - System Update / observability: Unit Test (Sub-Epic 8)

## Overview
- **Task ID**: `T-01975`
- **Component**: `aiosh-core::system_update_observability`
- **Objective**: Implement comprehensive Rust unit tests covering `SystemUpdateObservabilityReport` generation, telemetry text sanitization, policy evaluation integration, and health status computation (invariants `UOBS1..UOBS6`).

## Test Suite Design
The test suite `code/aiosh-rust/aiosh-core/tests/test_system_update_observability.rs` implements 7 unit tests:
1. `test_uobs1_default_report_generation`: Validates default report generated from clean `SystemUpdateService` state with slot A active, idle state, 0% progress, healthy status, and no policy.
2. `test_uobs2_report_with_active_manifest_and_staged_payload`: Validates report accuracy when artifacts are staged in the filesystem, confirming payload byte counting and manifest metadata reflection.
3. `test_uobs3_report_with_security_policy_evaluated`: Validates non-mutating policy evaluation integration, including default "not_evaluated" verdict when no manifest is active and "allow" verdict when manifest complies with policy rules.
4. `test_uobs4_telemetry_sanitization`: Verifies invariant `UOBS4` preventing log injection by stripping ASCII control characters, trimming whitespace, and truncating strings exceeding 256 characters across all telemetry fields.
5. `test_uobs5_health_computation`: Validates invariant `UOBS6` across state combinations (slot A vs slot B success flags, failed update state).
6. `test_uobs6_json_serialization_roundtrip`: Verifies canonical JSON serialization and deserialization fidelity (`serde_json`).
7. `test_uobs7_progress_clamping`: Validates progress percentages are strictly clamped to $[0, 100]$.
