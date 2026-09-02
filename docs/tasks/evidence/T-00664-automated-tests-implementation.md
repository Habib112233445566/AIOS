# T-00664 — Repository Health / automated tests: Implementation

## 1. Implementation Scope
Implemented all `check_h1` through `check_h7` functions in `tools/test_repo_health_suites.py`.

## 2. Deliverables
- **H1**: Runs `cargo test --lib repo_health::tests` and asserts `test result: ok`.
- **H2**: Runs `cargo test --lib repo_health_service::tests` for git hygiene.
- **H3**: Runs service tests confirming file bounds scanner correctness.
- **H4**: Runs `tools/check_security_policy.py` and asserts S1..S5 pass.
- **H5**: Runs `code/aiosh-cli/tests/test_repo_cli_smoke.py` and asserts 5/5 pass.
- **H6**: Runs `code/aiosh-mcp/tests/test_repo_mcp_smoke.py` and asserts 3/3 pass.
- **H7**: Runs `cargo test --lib repo_health_config::tests` and `test_repo_config_smoke.py`.

## 3. Test Verification Output
```text
[+] H1 data model integrity
[+] H2 git tree hygiene diagnostics
[+] H3 file bounds scanner
[+] H4 security governance audit
[+] H5 CLI surface commands
[+] H6 MCP tool schemas & JSON-RPC
[+] H7 configuration schema & hardening

PASS: repo_health_suites criteria (H1..H7)
```
