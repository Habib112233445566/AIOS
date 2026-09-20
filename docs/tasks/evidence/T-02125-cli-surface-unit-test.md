# T-02125: CLI Surface Unit Tests — PEP Decision Engine

## Overview
- **Task ID**: `T-02125`
- **Sub-Epic**: 3 (CLI Surface)
- **Status**: Completed

## Test Coverage & Execution Results

### 1. Rust Unit Tests (`code/aiosh-rust/aiosh-cli/src/main.rs:pep_cli_tests`)
Command: `cargo test -p aiosh-cli pep_cli_tests`
Output:
```
running 4 tests
test pep_cli_tests::test_pep_cli_help_and_unknown ... ok
test pep_cli_tests::test_pep_cli_path_hygiene ... ok
test pep_cli_tests::test_pep_cli_validation_and_hardening ... ok
test pep_cli_tests::test_pep_cli_rule_lifecycle_and_evaluation ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 39 filtered out; finished in 1.22s
```

### 2. Python End-to-End CLI Smoke Tests (`code/aiosh-cli/tests/test_pep_cli_smoke.py`)
Command: `python code/aiosh-cli/tests/test_pep_cli_smoke.py`
Output:
```
PASS: aiosh pep --help
PASS: aiosh pep unknown_cmd returns 2
PASS: aiosh pep path hygiene enforcement
PASS: aiosh pep lifecycle and evaluation
=== All PEP CLI tests passed ===
```

## Assertions Verified
1. **Help & Unknown Subcommands**: `--help` returns code `0` with command list; unknown commands return code `2` with structured error and failure audit row.
2. **Path Hygiene**: Non-JSON extensions and `..` path traversal sequences in `--store` are rejected with code `2`.
3. **Full Rule & Evaluation Lifecycle**:
   - Initial evaluate without rules returns exit code `1` (Deny).
   - `rule-add` with invalid effect returns code `2`.
   - `rule-add` with `permit` adds rule and persists atomically (code `0`).
   - `evaluate` matching request returns exit code `0` (Permit).
   - `evaluate` non-matching action returns exit code `1` (Deny).
   - `rule-list` lists rules and supports `--json` formatting.
   - `rule-remove` removes rule (code `0`); second removal returns code `1` (not found).
   - `evaluate` after rule removal returns code `1` (Deny).
4. **Hardening & Input Validation**: Missing required arguments (`--subject`, `--resource`, `--action`), resource traversal, and invalid rule IDs all return code `2`.
