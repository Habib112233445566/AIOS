# T-00662 — Repository Health / automated tests: Specification

## 1. Specification Overview
`tools/test_repo_health_suites.py` is a standalone, stdlib-only Python test runner covering criteria H1..H7 for the Repository Health subsystem.

## 2. Test Criteria Contract

### H1 — Data Model Integrity
- Run `cargo test --lib repo_health::tests` and assert all pass.
- Verify JSON roundtrip of `RepoHealthReport` via Rust unit tests.

### H2 — Git Tree Hygiene
- Run `cargo test --lib repo_health_service::tests` and assert git-related tests pass.

### H3 — File Bounds Scanner
- Verify file bounds checks pass via the core service test suite.

### H4 — Security Governance
- Run `python tools/check_security_policy.py` and assert exit 0, criteria S1..S5.

### H5 — CLI Surface
- Run `python code/aiosh-cli/tests/test_repo_cli_smoke.py` and assert exit 0, 5/5 tests.

### H6 — MCP Tool Interface
- Run `python code/aiosh-mcp/tests/test_repo_mcp_smoke.py` and assert exit 0, 3/3 tests.

### H7 — Configuration & Hardening
- Run `cargo test --lib repo_health_config::tests` and assert 3/3 pass.
- Run `python code/aiosh-cli/tests/test_repo_config_smoke.py` and assert exit 0.

## 3. Runner Contract
- Exit code 0: all H1..H7 pass.
- Exit code 1: any criterion fails.
- Emits `[+] H<n> <description>` per criterion on success.
- Emits `[-] H<n> <description>` and prints stderr on failure.
- Final summary: `PASS: repo_health_suites criteria (H1..H7)` or `FAIL: ...`.
