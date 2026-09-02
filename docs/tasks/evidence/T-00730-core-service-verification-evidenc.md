# T-00730 — Secrets & Access Hygiene / core service: Verification & Evidence

## 1. Verification Deliverables
- Implemented core service functions in `code/aiosh-rust/aiosh-core/src/secrets_service.rs`:
  - `scan_file_for_secrets`: Scans individual files against private keys (`SEC-001`), AWS Access Keys (`SEC-002`), GitHub PATs (`SEC-003`), Generic API tokens (`SEC-004`), and embedded config passwords (`SEC-005`), skipping binary content.
  - `scan_workspace_for_secrets`: Recursively traverses directory trees ignoring standard build/vcs folders (`.git`, `target`, `node_modules`, `.venv`, `dist`) and produces validated `SecretScanReport` objects.
- Unit test suite in `secrets_service::tests` passing 7/7 (clean file, private key, AWS+GitHub keys, password in config, binary skip, workspace scan, missing directory error).
- Extended test runner `tools/test_secrets_suites.py` validating criteria `K1..K4`.
- Updated reference documentation in `docs/README.md` under `## Secrets & Access Hygiene (T-00711..T-00810)`.

## 2. Test Execution & Evidence Log
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
[+] K1 data model integrity
[+] K2 private key scanner
[+] K3 API token scanner
[+] K4 config & env credentials scanner

PASS: secrets_suites criteria (K1..K4)
[+] E1 directory-health: found 1590 evidence files
[+] E2 ledger-consistency: verified 50 sampled completed tasks
[+] E3 file-bounds: all 1590 files bounded and valid UTF-8
[+] E4 hash-consistency: deterministic SHA-256 verified

PASS: evidence integrity criteria (E1..E4)
```
