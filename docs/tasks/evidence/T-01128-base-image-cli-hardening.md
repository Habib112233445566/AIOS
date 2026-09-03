# T-01128 — Base Image Build / CLI Surface: Hardening

**Date:** 2026-09-03
**Type:** Hardening
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / CLI Surface

## 1. Hardening Deliverables
- Enforced strict ASCII printable graphic validation on all target `<id>` parameters across `aiosh image show` and `aiosh image plan`.
- Prevented injection of control characters (e.g. `\x07`, `\x00`, escape codes) with exit code 2.
- Updated `test_cmd_image_flow` unit test to verify rejection of malformed control-character identifiers for both `show` and `plan`.

## 2. Test Execution Output
```
running 1 test
test task_cli_tests::test_cmd_image_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.34s
```
