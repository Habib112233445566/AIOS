# T-00535 — Evidence & Audit Trail / CLI surface: Unit Test

## 1. Unit Test Scope
This task tests the CLI command suite for Evidence & Audit Trail (`aiosh evidence`) covering happy paths, negative error states, missing argument validation, task filtering, and JSON output envelopes.

## 2. Test Cases & Coverage
1. `test_evidence_hash_prose`:
   - Computes SHA-256 in prose mode and asserts `[+] <path> -> <hash>` output format.
2. `test_evidence_hash_json`:
   - Validates JSON envelope output with `"subcommand": "evidence hash"` and 64-character SHA-256 hash.
3. `test_evidence_hash_missing_file_error`:
   - Asserts non-existent file produces exit code 1 with `"ok": false` error envelope.
4. `test_evidence_hash_missing_arg_error`:
   - Asserts missing positional argument produces exit code 2 and usage message.
5. `test_evidence_verify_default`:
   - Executes evidence verification against default manifest returning exit 0 and valid report.
6. `test_evidence_scan`:
   - Scans evidence directory and returns all task evidence artifacts.
7. `test_evidence_scan_filtered`:
   - Filters evidence records by specific task ID (`--task 501`).
8. `test_evidence_unknown_subcommand`:
   - Asserts invalid subcommand produces exit code 2.

## 3. Test Execution Output
```text
PASS: aiosh evidence hash prose
PASS: aiosh evidence hash --json
PASS: aiosh evidence hash missing file error
PASS: aiosh evidence hash missing arg error
PASS: aiosh evidence verify --json
PASS: aiosh evidence scan --json
PASS: aiosh evidence scan filtered by task
PASS: aiosh evidence unknown subcommand error
All 8 evidence CLI unit and smoke tests passed successfully!
```
