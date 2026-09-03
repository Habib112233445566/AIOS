# T-01155 — Base Image Build / Automated Tests: Unit Test

**Date:** 2026-09-03
**Type:** Unit Test
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Automated Tests

## 1. Unit Test Deliverables
- Executed unit test suite across all `aiosh-core::base_image*` modules:
  - `base_image::tests` (13 tests): Invariants I1..I6, SemVer, naming, and size bounds.
  - `base_image_service::tests` (9 tests): Invariants P1..P4, canonical store, 4-stage plans, and persistence.
  - `base_image_config::tests` (3 tests): Invariants CF1..CF6, config cascading, and roundtrip serialization.
  - Integration suite `test_base_image_automated` (6 tests): Determinism, scale, override, e2e cohesion, rejections, and parity.

## 2. Test Execution Output
```
running 25 tests
test base_image::tests::test_cmdline_hardening ... ok
test base_image::tests::test_canonical_manifest_debian_valid ... ok
test base_image::tests::test_canonical_manifest_alpine_valid ... ok
test base_image::tests::test_artifact_sha256_validation ... ok
test base_image::tests::test_empty_packages ... ok
test base_image::tests::test_hostname_hardening ... ok
test base_image::tests::test_image_format_display ... ok
test base_image::tests::test_invalid_filesystem ... ok
test base_image::tests::test_invalid_manifest_id ... ok
test base_image::tests::test_invalid_semver ... ok
test base_image::tests::test_json_roundtrip ... ok
test base_image::tests::test_package_name_hardening ... ok
test base_image::tests::test_size_budget_limits ... ok
test base_image_config::tests::test_default_config_valid ... ok
test base_image_config::tests::test_validation_cf1_cf6_failures ... ok
test base_image_service::tests::test_build_plan_validation_failures ... ok
test base_image_service::tests::test_build_plan_alpine_tarball ... ok
test base_image_service::tests::test_duplicate_image_registration_rejected ... ok
test base_image_service::tests::test_generate_build_plan_nonexistent_image ... ok
test base_image_service::tests::test_generate_build_plan_valid ... ok
test base_image_service::tests::test_image_store_canonical_initialization ... ok
test base_image_service::tests::test_image_store_filters ... ok
test base_image_config::tests::test_persistence_roundtrip ... ok
test base_image_service::tests::test_image_store_persistence_roundtrip ... ok
test base_image_service::tests::test_oversized_store_file_rejected ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 246 filtered out; finished in 0.04s
```
