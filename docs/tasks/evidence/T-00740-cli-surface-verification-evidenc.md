# T-00740 — Secrets & Access Hygiene / CLI surface: Verification & Evidence

## 1. Verification Deliverables
- Fully implemented CLI subcommand handler `cmd_secrets` in `code/aiosh-rust/aiosh-cli/src/main.rs`:
  - `aiosh secrets scan [--repo <path>] [--file <path>] [--json] [--max-bytes <n>]`
  - `aiosh secrets check [--repo <path>] [--json]`
- Unit tests in `aiosh-cli::task_cli_tests::test_cmd_secrets_scan_and_check` passing 16/16.
- Standalone test runner `tools/test_secrets_suites.py` validating criteria `K1..K5`.
- Updated operator reference documentation in `docs/README.md` under `## Secrets & Access Hygiene (T-00711..T-00810)`.

## 2. Test Execution & Evidence Log
```text
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
[+] K1 data model integrity
[+] K2 private key scanner
[+] K3 API token scanner
[+] K4 config & env credentials scanner
[+] K5 CLI surface commands & options

PASS: secrets_suites criteria (K1..K5)
[+] E1 directory-health: found 1621 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1621 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)
```
