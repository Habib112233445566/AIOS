# T-01175 — Base Image Build / Observability: Unit Test

**Date:** 2026-09-04
**Type:** Unit Test
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Observability

## 1. Test Suite Summary
- Implemented `code/aiosh-rust/aiosh-core/tests/test_base_image_observability.rs` validating:
  - `test_ob1_ob2_ob3_categorical_breakdowns`: verifies format, architecture, and distro breakdown sums equal total images.
  - `test_ob4_policy_compliance_tracking`: verifies compliance counts under Enforcing and Permissive policies.
  - `test_ob5_size_budget_and_averages`: verifies size budget calculations on populated and empty stores.
  - `test_kernel_version_aggregation`: verifies distinct kernel version extraction.
  - `test_synthetic_scale_and_negative_invariants`: verifies 25 synthetic manifests and validates detection of arithmetic tampering in `validate()`.
- All tests pass: `test result: ok. 5 passed; 0 failed`.
