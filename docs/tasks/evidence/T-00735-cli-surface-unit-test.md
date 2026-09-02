# T-00735 — Secrets & Access Hygiene / CLI surface: Unit Test

## 1. Test Deliverables
- Added unit tests in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - `task_cli_tests::test_cmd_secrets_scan_and_check`: Asserts return codes for `aiosh secrets scan`, `aiosh secrets scan --json`, `aiosh secrets check`, `aiosh secrets check --json`, and error handling on invalid subcommands.
- Added criteria `K5` (`test_k5_cli_surface`) to `tools/test_secrets_suites.py`.
- Verified execution of test suite via standalone runner.

## 2. Test Execution Output
```text
[+] K1 data model integrity
[+] K2 private key scanner
[+] K3 API token scanner
[+] K4 config & env credentials scanner
[+] K5 CLI surface commands & options

PASS: secrets_suites criteria (K1..K5)
```
