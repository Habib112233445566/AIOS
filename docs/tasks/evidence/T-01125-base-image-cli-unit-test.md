# T-01125 — Base Image Build / CLI Surface: Unit Test

**Date:** 2026-09-03
**Type:** Unit Test
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / CLI Surface

## 1. Unit Test Deliverables
- Implemented and verified `test_cmd_image_flow` in `aiosh-cli::task_cli_tests`.
- Asserted exit code contracts across all branches:
  - `list`: exit 0 in both table and JSON modes
  - `show`: exit 0 for existing image, exit 1 for missing image, exit 2 for missing argument
  - `plan`: exit 0 for existing image, exit 1 for missing image, exit 2 for missing argument
  - `filter`: exit 0 for valid format query, exit 2 for invalid format option
  - `--help`: exit 0
  - unknown subcommand: exit 2
- Test passed cleanly with zero regressions.

## 2. Test Execution Output
```
running 1 test
test task_cli_tests::test_cmd_image_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 2.65s
```
