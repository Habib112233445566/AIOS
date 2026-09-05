# T-01225: Package Management - CLI Surface: Unit Tests

## Metadata
- **Task ID:** `T-01225`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Package Management CLI Surface Unit Tests
- **Status:** Complete

## 1. Unit Test Deliverables
- Expanded unit test coverage in `code/aiosh-rust/aiosh-cli/src/main.rs` (`test_cmd_package_flow`):
  - **`validate`**:
    - Valid name (`curl`) -> Exit code 0
    - Valid name with `--json` -> Exit code 0
    - Invalid name (`Curl`) -> Exit code 2
    - Valid `PackageSpec` JSON -> Exit code 0
    - Invalid `PackageSpec` JSON (uppercase name) -> Exit code 2
    - Missing flags -> Exit code 2
  - **`list`**:
    - Default enumeration -> Exit code 0
    - JSON formatting with `--format deb` filter -> Exit code 0
    - Invalid format parameter (`invalid_fmt`) -> Exit code 2
    - Invalid state parameter (`invalid_state`) -> Exit code 2
  - **`show`**:
    - Existing package (`curl`) -> Exit code 0
    - Non-existent package (`nonexistent`) -> Exit code 2
  - **`search`**:
    - Existing pattern (`curl`) -> Exit code 0
    - JSON search with pattern (`lib`) -> Exit code 0
    - Missing search pattern argument -> Exit code 2
  - **`plan`**:
    - Valid actions batch -> Exit code 0
    - Dry-run mode with `--json` -> Exit code 0
    - Missing dependency rejection -> Exit code 2
  - **`apply`**:
    - Missing `--actions` or `--plan` arguments -> Exit code 2
    - Dry-run transaction execution via `--actions` -> Exit code 0
    - Dry-run transaction execution via `--plan` -> Exit code 0
    - Real transaction application with persistent disk state roundtrip -> Exit code 0
    - Malformed JSON handling -> Exit code 2
    - Dependency closure violation rejection -> Exit code 2
  - **`--help` & Unknown**:
    - `--help` usage display -> Exit code 0
    - Unknown subcommand -> Exit code 2

## 2. Test Execution Output
```text
running 1 test
test task_cli_tests::test_cmd_package_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.64s
```
