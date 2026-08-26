# T-00127 — CI Smoke Orchestration / core service: Security Review

**Date:** 2026-08-25
**Feature:** `tools/ci_service.py` (Read-only summary consumer)

## 1. Input Validation & Untrusted Content Handling
- **Path Injection:** The CLI allows arbitrary file paths via the `--file` argument or `AIOSH_CI_RESULTS` environment variable. `open(path)` is called without sanitization. However, because the tool strictly expects the target file to be valid JSON (and further enforces specific schema keys), reading a sensitive file like `/etc/shadow` simply throws a `ValueError` during parsing and does not leak the file's contents in the error message. 
- **Memory Exhaustion (DoS):** `json.load(f)` reads the entire file into memory. A malicious or pathologically large JSON artifact could cause an Out-Of-Memory (OOM) error. Because `ci_service.py` operates as a CI post-processor, an OOM acts as a fail-closed CI failure (exit code 2 or crash), which is an acceptable boundary state.

## 2. PEP Gating & Audit Row Emission
- **State Changes:** None. The service is strictly read-only by contract (T-00122 §8).
- **PEP & Audit:** Because it does not mutate system state, create grants, or execute dangerous tools, PEP gating and audit-row emissions are explicitly not required for this component.

## 3. Abuse Scenarios
1. **Summary Forgery:** A malicious test suite could attempt to overwrite `/tmp/aiosh-ci-results.json` directly to fabricate an `all_pass=True` result before `ci_run.py` finishes. *Mitigation:* `ci_run.py` writes atomically using a temporary file and `os.replace` after all suites finish, clobbering any forged files.
2. **Pathological Log Volume:** A malicious test suite outputs gigabytes of logs to exhaust disk space and crash the parser. *Mitigation:* `ci_run.py` caps log reads (T-00118 bounded tail read limit of 64KB), meaning the JSON artifact produced is inherently bounded in size.

## 4. Conclusion
- **Policy Bypasses:** None found.
- **Verdict:** PASS. No blocking notes or fixes required.
