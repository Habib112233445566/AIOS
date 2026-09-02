# T-00668 — Repository Health / automated tests: Hardening

## 1. Hardening Overview
Verifies the test runner `tools/test_repo_health_suites.py` handles failures gracefully.

## 2. Hardening Measures
- **Timeout Enforcement**: All `_run()` subprocess calls use `timeout=120` seconds.
- **Exception Handling**: The `main()` dispatcher catches all exceptions from `check_h*` functions and emits `[-]` prefixed failure lines without crashing.
- **Exit Code Semantics**: Returns exit code 0 only when ALL criteria pass; returns 1 on any failure.
- **No Resource Leaks**: The test runner opens no files, creates no temporary directories, and holds no database connections. All subprocess handles are cleaned up by `subprocess.run()`.
- **Deterministic Output**: Each criterion emits exactly one `[+]` or `[-]` line, enabling grep-based CI parsing.
