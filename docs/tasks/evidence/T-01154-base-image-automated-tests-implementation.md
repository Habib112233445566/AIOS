# T-01154 — Base Image Build / Automated Tests: Implementation

**Date:** 2026-09-03
**Type:** Implementation
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Automated Tests

## 1. Implementation Deliverables
- Implemented full suite in `code/aiosh-rust/aiosh-core/tests/test_base_image_automated.rs`:
  - `test_t1_build_plan_determinism`: 50-iteration invariant and plan field stability test.
  - `test_t2_registry_stress_and_bulk_query`: Multi-manifest batch registration and lookup test.
  - `test_t3_configuration_override_resolution`: Precedence testing between file, env, and defaults.
  - `test_t4_end_to_end_pipeline_cohesion`: End-to-end serialization, reload, and plan generation test.
  - `test_t5_invalid_manifest_rejections`: Rejection of malformed SemVer, bad package names, oversized size budget, and unsafe kernel cmdline.
  - `test_t6_mcp_and_cli_parity`: Parity between plan stage names and expected system flow.

## 2. Test Execution Output
```
running 6 tests
test test_t1_build_plan_determinism ... ok
test test_t2_registry_stress_and_bulk_query ... ok
test test_t3_configuration_override_resolution ... ok
test test_t4_end_to_end_pipeline_cohesion ... ok
test test_t5_invalid_manifest_rejections ... ok
test test_t6_mcp_and_cli_parity ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
```
