# T-02425: Sandbox Enforcement CLI Surface Unit Test

## 1. Overview
This document records verification evidence for task T-02425: Sandbox Enforcement CLI Surface Unit Test.
Automated unit tests covering valid inputs, negative edge cases, parameter boundaries, and output formats for `aiosh sandbox` were implemented and executed.

## 2. Test Suites & Coverage
Unit testing was authored in two complementary suites:
1. **Rust In-Tree Unit Tests** (`code/aiosh-rust/aiosh-cli/src/main.rs:sandbox_cli_tests`):
   - `test_sandbox_cli_help_and_subcommands`: Verifies help usage and unknown subcommand handling.
   - `test_sandbox_cli_profiles_and_probe`: Verifies profile listing and capability probing in text and JSON formats.
   - `test_sandbox_cli_exec_validation`: Verifies rejection of missing `--` delimiters, missing binaries, directory traversal (`..`) in `--cwd`, and nonexistent profiles.
   - `test_sandbox_cli_exec_success`: Verifies end-to-end command execution and return codes.
2. **Python Integration/Unit Tests** (`code/aiosh-cli/tests/test_sandbox_cli.py`):
   - `test_sandbox_help`: Validates `aiosh sandbox` without args, `--help`, `-h`, and alias `aiosh sb`.
   - `test_sandbox_profiles`: Validates standard profile listing in human tabular mode and `--json` envelope.
   - `test_sandbox_probe`: Validates kernel capability detection output in human text and `--json`.
   - `test_sandbox_exec_validation`: Tests missing delimiter, missing binary, path traversal in `--cwd`, and unknown profiles (all exit code 2).
   - `test_sandbox_exec_success`: Tests successful command execution in text mode and structured JSON envelope format (`exit_code = 0`).
   - `test_sandbox_unknown_subcommand`: Asserts exit code 2 and structured error envelope for unmapped subcommands.

## 3. Execution Results
```text
$ python -m pytest code/aiosh-cli/tests/test_sandbox_cli.py
============================== 6 passed in 1.02s ==============================
```
All unit tests executed standalone and passed cleanly.
