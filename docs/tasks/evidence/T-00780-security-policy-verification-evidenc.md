# T-00780 — Secrets & Access Hygiene / security policy: Verification & Evidence

## 1. Verification Deliverables
- Fully validated `SECURITY.md` incorporating Secrets & Access Hygiene policy and disclosure requirements.
- Linked security review evidence in `docs/tasks/evidence/T-00777-security.md`.
- Automated test suites passing:
  - `tools/check_security_policy.py` (S1..S5 PASS).
  - `tools/check_task_docs.py` (C1..C6 PASS).
  - `tools/test_secrets_suites.py` (K1..K7 PASS).
  - `tools/check_evidence.py` (E1..E4 PASS).

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
[+] K6 MCP tool schemas & execution
[+] K7 SecretsConfig schema, validation & roundtrip

PASS: secrets_suites criteria (K1..K7)
[+] E1 directory-health: found 1745 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1745 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)
```
