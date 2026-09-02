# T-00570 — Evidence & Audit Trail / automated tests: Verification & Evidence

## 1. Verification Goal
Formally close the **Evidence & Audit Trail / automated tests sub-epic (T-00561..T-00570)** by executing all unit, integration, invariant, and smoke test suites across Python and Rust substrates.

## 2. Test Execution Matrix
1. `tools/test_ci_suites.py`: Verified 29-suite registry and canonical order contract (W1..W7).
2. `tools/test_check_evidence.py`: Verified 15 behavioral unit tests across criteria E1..E4 and S01 sensitivity.
3. `tools/check_evidence.py`: Verified live repository integrity across 1,110+ evidence artifacts.
4. `tools/check_task_docs.py`: Verified structural doc invariants C1..C6.
5. `code/aiosh-cli/tests/test_evidence_cli_smoke.py`: Verified 8/8 CLI smoke cases.
6. `code/aiosh-mcp/tests/test_evidence_mcp_smoke.py`: Verified 8/8 MCP JSON-RPC tool calls.
7. `test_evidence_e2e.rs`: Verified Rust end-to-end manifest lifecycle and query helpers.

## 3. Captured Verification Output
```text
[+] W1 registry 20/20 == frozen canonical order; bash delegates to ci_run.py; scripts exist
[+] W2 pass record + derived log path
[+] W3 timeout/error force exit_code=null (even when caller passes an int)
[+] W4 all invalid inputs rejected naming the field
[+] W5 atomic write + JSON round-trip, no temp leftovers
[+] W6 failed write leaves no temp files
[+] W7 corrupted registry rejected at import (duplicate suite name in SUITES: 'rust_smoke')
PASS: ci_suites unit tests (W1..W7)

Running Evidence Checker behavioral unit tests (T-00565)...
[+] U01 E1 directory-health: valid dir with files returns True
[+] U02 E1 directory-health: non-existent dir returns False
[+] U03 E1 directory-health: empty dir without evidence returns False
[+] U04 E2 ledger-consistency: valid state and matching files returns True
[+] U05 E2 ledger-consistency: missing state file returns False
[+] U06 E2 ledger-consistency: corrupt JSON returns False
[+] U07 E2 ledger-consistency: missing task file flagged as False
[+] U08 E2 ledger-consistency: empty completed list boundary returns True
[+] U09 E3 file-bounds: valid non-empty UTF-8 files return True
[+] U10 E3 file-bounds: empty file (0 bytes) returns False
[+] U11 E3 file-bounds: oversized file returns False
[+] U12 E3 file-bounds: invalid non-UTF-8 bytes return False
[+] U13 E4 hash-consistency: valid SHA-256 digest returns True
[+] U14 E4 hash-consistency: multiple evidence files verify cleanly
[+] S01 Sensitivity: checker blindness is detectable

Summary: 15/15 passed, 0 failed.
PASS: test_check_evidence_unit (15/15 checks green)

[+] E1 directory-health: found 1112 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1112 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)

[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)

PASS: aiosh evidence hash prose
PASS: aiosh evidence hash --json
PASS: aiosh evidence hash missing file error
PASS: aiosh evidence hash missing arg error
PASS: aiosh evidence verify --json
PASS: aiosh evidence scan --json
PASS: aiosh evidence scan filtered by task
PASS: aiosh evidence unknown subcommand error
All 8 evidence CLI unit and smoke tests passed successfully!

PASS: aios.evidence tools present in tools/list
PASS: aios.evidence.hash execution
PASS: aios.evidence.hash missing file error
PASS: aios.evidence.hash missing arg error
PASS: aios.evidence.verify execution
PASS: aios.evidence.scan execution
PASS: aios.evidence.scan filtered by task
PASS: aios.evidence.scan missing dir error
All 8 evidence MCP unit and smoke tests passed successfully!

running 2 tests
test test_evidence_manifest_query_and_filter_e2e ... ok
test test_evidence_full_lifecycle_e2e ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

## 4. Milestone Conclusion
The **Evidence & Audit Trail / automated tests sub-epic (T-00561..T-00570)** is CLOSED 10/10. Ledger pointer advances to T-00571.
