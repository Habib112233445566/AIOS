# T-01557 — Filesystem Layout automated tests: Security Review

## Metadata
- **Task ID:** `T-01557`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / automated tests
- **Status:** Complete — comprehensive security review of the automated test suite (`test_fs_layout_automated_cases.py` and `test_fs_layout_suites.py`); zero policy bypasses found.
- **Date:** 2026-09-19
- **Depends on:** `T-01556` (Automated Tests Integration)
- **Feeds:** `T-01558` (Automated Tests Hardening)
- **Artifacts:** `docs/tasks/evidence/T-01557-automated-tests-security-review.md`, `docs/tasks/evidence/T-01557-security.md`

---

## 1. Scope & Security Review Objectives

This security review evaluates the automated testing infrastructure for Filesystem Layout (`test_fs_layout_automated_cases.py` and `test_fs_layout_suites.py`), focusing on:
1. Subprocess command invocation safety (CWE-78 command injection).
2. Temporary directory and path confinement (CWE-22 path traversal).
3. Store corruption and fault recovery (fail-closed handling).
4. Audit trail isolation (`AIOSH_HOME` confinement to prevent operator log contamination).

---

## 2. Abuse Scenarios & Verification Matrix

| # | Abuse Scenario | Attack Vector / Malicious Payload | Defense Mechanism & Observed Behavior | Verdict |
|---|---|---|---|---|
| **S1** | **Shell Metacharacter Injection** | Malicious layout ID or path containing `; rm -rf /` or `& calc.exe` passed as CLI argument | Subprocess invocation passes argument lists directly to the OS kernel without invoking a shell (`shell=False`). Characters are treated as literal arguments. | **PASS** |
| **S2** | **Cross-Test Store Pollution** | Tests modifying shared default store locations, corrupting concurrent or subsequent test runs | Each test suite runs against an isolated `tempfile.TemporaryDirectory` with per-test store files (`store_a1.json`, etc.). No persistent state is shared. | **PASS** |
| **S3** | **Operator Audit Log Tampering** | Test operations writing rows into the operator's real `~/.aios/audit.db` | Tests requiring audit assertions explicitly isolate `AIOSH_HOME` to an ephemeral directory. Production audit logs remain completely untouched. | **PASS** |
| **S4** | **Corrupted Store Overwrite / Truncation** | Attacker damages store JSON; subsequent tool invocation blindly overwrites or truncates the file | CLI refuses unparseable store with exit code 1; atomic replacement is skipped; corrupt file is preserved for analysis. | **PASS** |
| **S5** | **Subprocess Denial of Service / Hang** | An external command blocks indefinitely (e.g. reading from an open FIFO or slow pipe) | Every subprocess invocation enforces an explicit 60-second timeout in the test harness and 180-second timeout in the aggregate runner. | **PASS** |

---

## 3. Findings & Hardening Recommendations (T-01558)

- **Vulnerabilities Found:** 0 (Zero).
- **Policy Bypasses Open:** None.
- **Recommendations:**
  1. Ensure all temporary directories are cleaned up even if assertions fail by wrapping test execution in context managers (`tempfile.TemporaryDirectory`).
  2. Maintain strict 60-second timeouts across all subprocess calls.
  3. Keep `shell=False` as an immutable invariant across all test scripts.

---

## 4. Acceptance Confirmation

- [x] Security evidence file exists with abuse scenarios.
- [x] No known policy bypass remains open.
