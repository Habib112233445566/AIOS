# T-01118 — Base Image Build / Core Service: Hardening

**Date:** 2026-09-03
**Type:** Hardening
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Core Service

## 1. Hardening Deliverables
- **10 MiB File Size Ceiling**: Enforced strict 10 MiB limit on image store file loading in `load_from_path` with stream read cap.
- **Unix Permission Hardening**: Configured file creation permissions (`0o644`) when persisting image stores.
- **Unit Test Coverage**: Added `test_oversized_store_file_rejected` verifying that stores $>10 \text{ MiB}$ are rejected with honest error diagnostics.
- All 9 unit tests passing cleanly in `base_image_service::tests`.

## 2. Test Execution Output
```
running 9 tests
test base_image_service::tests::test_build_plan_alpine_tarball ... ok
test base_image_service::tests::test_build_plan_validation_failures ... ok
test base_image_service::tests::test_duplicate_image_registration_rejected ... ok
test base_image_service::tests::test_generate_build_plan_nonexistent_image ... ok
test base_image_service::tests::test_generate_build_plan_valid ... ok
test base_image_service::tests::test_image_store_canonical_initialization ... ok
test base_image_service::tests::test_image_store_filters ... ok
test base_image_service::tests::test_image_store_persistence_roundtrip ... ok
test base_image_service::tests::test_oversized_store_file_rejected ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 259 filtered out; finished in 0.12s
```
