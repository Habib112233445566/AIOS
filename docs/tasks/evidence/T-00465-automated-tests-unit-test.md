# T-00465 — Documentation Index Control / automated tests: Unit Test

## 1. Unit Test Scope
This task implements standalone behavioral unit tests in `tools/test_doc_index_unit.py` testing `test_doc_index_suites.py` criteria D1..D7, individual assertion functions, and runner sensitivity proofs.

## 2. Test Cases & Coverage
- **U01/U02**: D1 Manifest valid serialization & query filtering.
- **U03/U04**: D1 Manifest negative query filtering & `check_d1_manifest_model` happy path.
- **U05/U06**: D2 Configuration resolution & oversized config (>64 KiB) detection.
- **U07/U08**: D3 Markdown H1 title parsing & inline link extraction excluding external URLs.
- **U09/U10**: D3/D4 Link extraction and in-tree link validation checks.
- **U11/U12**: D5 CLI subcommand execution & D6 MCP JSON-RPC tool dispatch checks.
- **U13**: D7 Hardening limits and negative input verification.
- **S01**: Sensitivity proof verifying that when a criteria checker fails, `run_all_criteria` detects failure and reports non-zero exit.

## 3. Test Execution Output
```text
[+] U01: D1 manifest valid serialization
[+] U02: D1 query filtering
[+] U03: D1 negative query match
[+] U04: D1 check function succeeds
[+] U05: D2 check function succeeds
[+] U06: D2 oversized config detected
[+] U07: D3 title H1 extraction
[+] U08: D3 inline relative link extraction excluding external URLs
[+] U09: D3 check function succeeds
[+] U10: D4 check function succeeds
[+] U11: D5 CLI subcommands check succeeds
[+] U12: D6 MCP surface check succeeds
[+] U13: D7 hardening limits check succeeds
[-] D1 manifest model & query helpers: intentional test failure
[+] D2 configuration hierarchy & limits
[+] D3 title parsing & link extraction
[+] D4 link integrity & traversal detection
[+] D5 CLI subcommand execution & json mode
[+] D6 MCP tool execution & protocol schemas
[+] D7 hardening limits & negative error bounds
[+] S01: Sensitivity proof -- failing checker causes test runner failure

PASS: all 14 doc_index unit tests green
```
