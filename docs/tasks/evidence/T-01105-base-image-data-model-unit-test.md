# T-01105 — Base Image Build / Data Model: Unit Test

**Date:** 2026-09-03
**Type:** Unit Test
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Data Model

## 1. Unit Test Deliverables
- Implemented and executed 10 unit tests for `base_image` covering:
  - Canonical Debian 12 minimal manifest validity
  - Canonical Alpine 3.19 container manifest validity
  - Rejection of invalid manifest identifiers
  - Rejection of invalid SemVer strings
  - Rejection of empty package sets
  - Rejection of unsupported filesystems
  - Size budget boundary assertions ($>0$ and $\le 10 \text{ GiB}$)
  - Lowercase hex SHA-256 validation (length 64, lowercase)
  - Full JSON roundtrip serialization
  - `ImageFormat` display formatting
- Zero failures or regressions.

## 2. Test Execution Output
```
running 10 tests
test base_image::tests::test_artifact_sha256_validation ... ok
test base_image::tests::test_canonical_manifest_alpine_valid ... ok
test base_image::tests::test_canonical_manifest_debian_valid ... ok
test base_image::tests::test_empty_packages ... ok
test base_image::tests::test_image_format_display ... ok
test base_image::tests::test_invalid_filesystem ... ok
test base_image::tests::test_invalid_manifest_id ... ok
test base_image::tests::test_invalid_semver ... ok
test base_image::tests::test_json_roundtrip ... ok
test base_image::tests::test_size_budget_limits ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 246 filtered out; finished in 0.15s
```
