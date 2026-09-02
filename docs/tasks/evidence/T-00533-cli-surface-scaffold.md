# T-00533 — Evidence & Audit Trail / CLI surface: Scaffold

## 1. Scaffold Scope
This task creates the initial CLI wiring for `aiosh evidence` and the automated smoke test script `code/aiosh-cli/tests/test_evidence_cli_smoke.py`.

## 2. Scaffold Contents
- Registered `aiosh evidence` command entry in `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Scaffolded `cmd_evidence` handler supporting `verify` and `hash` subcommands.
- Created `code/aiosh-cli/tests/test_evidence_cli_smoke.py` asserting prose and JSON execution.

## 3. Test Verification
```text
PASS: aiosh evidence hash prose
PASS: aiosh evidence hash --json
PASS: aiosh evidence verify --json
All evidence CLI smoke tests passed successfully!
```
