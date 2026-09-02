# T-00564 — Evidence & Audit Trail / automated tests: Implementation

## 1. Implementation Scope
This task implements automated end-to-end integration tests in `code/aiosh-rust/aiosh-core/tests/test_evidence_e2e.rs` and the CI validation tool `tools/check_evidence.py`.

## 2. Implementation Details
- `tools/check_evidence.py`:
  - E1 (directory-health): Asserts evidence directory exists and has files.
  - E2 (ledger-consistency): Validates that completed tasks have valid evidence files on disk.
  - E3 (file-bounds): Validates non-empty files, valid UTF-8 encoding, and 16 MiB size caps across 1000+ files.
  - E4 (hash-consistency): Checks deterministic SHA-256 calculation.
- `test_evidence_e2e.rs`:
  - `test_evidence_full_lifecycle_e2e`: Simulates 10-step sub-epic manifest construction, verification pass, content tampering detection, and file deletion detection.
  - `test_evidence_manifest_query_and_filter_e2e`: Tests `get_record` and `filter_by_step`.

## 3. Test Verification
```text
running 2 tests
test test_evidence_manifest_query_and_filter_e2e ... ok
test test_evidence_full_lifecycle_e2e ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s
```
```text
[+] E1 directory-health: found 1098 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1098 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)
```
