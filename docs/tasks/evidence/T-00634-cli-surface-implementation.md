# T-00634 — Repository Health / CLI surface: Implementation

## 1. Implementation Scope
This task implements `cmd_repo` in `code/aiosh-rust/aiosh-cli/src/main.rs`.

## 2. Implementation Deliverables
- Implemented `cmd_repo(args: &[String]) -> i32`:
  - Parses `health` and `check` subcommands.
  - Supports `--repo <path>` for target repository path and `--json` for machine output.
  - Generates ASCII diagnostic summaries and structured JSON.
  - Returns exit code 0 on Pass/Warn, 1 on Fail, and 2 on syntax errors.
  - Emits an audit row with tool `repo.health`.
- Added unit test `test_cmd_repo_health_and_check`.

## 3. Test Verification Output
```text
running 1 test
test task_cli_tests::test_cmd_repo_health_and_check ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 3.80s
```
