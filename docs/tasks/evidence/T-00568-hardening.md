# T-00568 — Evidence & Audit Trail / automated tests: Hardening

## 1. Hardening Scope
This task verifies and documents the hardening mechanisms applied across the Evidence & Audit Trail automated tests and CI verification harness.

## 2. Hardening Measures Implemented

### Timeouts & Asynchronous Safety:
- All test subprocesses execute with explicit wall-clock timeouts (15s–30s) preventing hung processes or zombie test runners.
- The CI orchestrator (`tools/ci_run.py`) wraps suite execution with process-group cleanup on timeout.

### IO & Memory Bounds:
- `MAX_DOC_BYTES = 16 * 1024 * 1024` (16 MiB max) enforced on all evidence verification paths.
- Chunked binary reads in 64 KiB buffers (`f.read(65536)`) prevent high memory consumption during SHA-256 calculation.
- Evidence scanning samples completed tasks in bounded batches, keeping execution fast and predictable as the ledger grows toward 10,000 tasks.

### Sandbox & Resource Isolation:
- `tools/test_check_evidence.py` uses `tempfile.mkdtemp` and `try...finally` teardown to ensure zero temporary directory leaks on success or error paths.
- Module attribute rebinding restores original paths on test completion.

### Error Envelope Uniformity:
- CLI commands return standardized exit codes (`0` for success, `1` for execution failure, `2` for grammar/usage errors).
- JSON modes emit structured error envelopes (`{ "ok": false, "error": "<reason>" }`) on failure.

## 3. Verification Output
```text
[+] E1 directory-health: found 1107 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1107 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)
PASS: test_check_evidence_unit (15/15 checks green)
PASS: ci_suites unit tests (W1..W7)
```
