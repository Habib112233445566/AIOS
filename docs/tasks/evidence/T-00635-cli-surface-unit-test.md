# T-00635 — Repository Health / CLI surface: Unit Test

## 1. Unit Test Scope
This task tests the CLI interface `aiosh repo` across stdout prose formatting, structured JSON output, alias resolution, custom path routing, and unknown command rejection.

## 2. Test Execution & Coverage
1. **`test_repo_health_prose`**:
   - Asserts `aiosh repo health` outputs formatted diagnostic tables with status indicators (`[+]`, `[!]`, `[-]`).
2. **`test_repo_health_json`**:
   - Asserts `aiosh repo health --json` emits valid JSON containing `overall_status`, `total_checks`, and `checks` list.
3. **`test_repo_check_alias`**:
   - Asserts `aiosh repo check` is a functional alias for `aiosh repo health`.
4. **`test_repo_custom_path`**:
   - Asserts `aiosh repo health --repo <path>` targets a specific directory.
5. **`test_repo_invalid_subcommand`**:
   - Asserts invalid subcommands return exit code 2 and print usage text.

## 3. Test Verification Output
```text
PASS: aiosh repo health prose output
PASS: aiosh repo health --json output
PASS: aiosh repo check alias
PASS: aiosh repo health --repo custom path
PASS: aiosh repo invalid subcommand rejection

ALL REPO CLI SMOKE TESTS PASSED!
```
