# T-01108 — Base Image Build / Data Model: Hardening

**Date:** 2026-09-03
**Type:** Hardening
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Data Model

## 1. Hardening Deliverables
- **Package Name Character Validation**: Enforced lowercase alphanumeric start and restricted characters (`[a-z0-9+.-]`, length 1..128).
- **RFC 1123 Hostname Validation**: Enforced length 1..63, character set `[a-z0-9-]`, no leading/trailing hyphens.
- **Kernel Cmdline Control Character Rejection**: Enforced length $\le 4096$ and prohibited null/newline characters.
- **Unit Test Coverage**: Verified with 13 unit tests in `base_image::tests`.

## 2. Test Execution Output
```
running 13 tests
test base_image::tests::test_canonical_manifest_alpine_valid ... ok
test base_image::tests::test_cmdline_hardening ... ok
test base_image::tests::test_canonical_manifest_debian_valid ... ok
test base_image::tests::test_artifact_sha256_validation ... ok
test base_image::tests::test_empty_packages ... ok
test base_image::tests::test_hostname_hardening ... ok
test base_image::tests::test_image_format_display ... ok
test base_image::tests::test_invalid_filesystem ... ok
test base_image::tests::test_invalid_manifest_id ... ok
test base_image::tests::test_invalid_semver ... ok
test base_image::tests::test_package_name_hardening ... ok
test base_image::tests::test_json_roundtrip ... ok
test base_image::tests::test_size_budget_limits ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 246 filtered out; finished in 0.02s
```
