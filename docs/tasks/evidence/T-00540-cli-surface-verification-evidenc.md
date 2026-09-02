# T-00540 — Evidence & Audit Trail / CLI surface: Verification & Evidence

## 1. Sub-Epic Verification Overview
This task concludes the Evidence & Audit Trail / CLI surface sub-epic (T-00531..T-00540), verifying that all command line tools (`aiosh evidence <verify|hash|scan>`), argument parsers, error envelopes, and automated smoke test suites pass green.

## 2. Test Execution Results

### A. CLI Smoke & Behavioral Unit Tests (`code/aiosh-cli/tests/test_evidence_cli_smoke.py`)
```text
PASS: aiosh evidence hash prose
PASS: aiosh evidence hash --json
PASS: aiosh evidence hash missing file error
PASS: aiosh evidence hash missing arg error
PASS: aiosh evidence verify --json
PASS: aiosh evidence scan --json
PASS: aiosh evidence scan filtered by task
PASS: aiosh evidence unknown subcommand error
All 8 evidence CLI unit and smoke tests passed successfully!
```

### B. Documentation Invariants (`tools/check_task_docs.py`)
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```

### C. Security Policy Invariants (`tools/check_security_policy.py`)
```text
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
```

## 3. Sub-Epic Milestone Closeout
- **Sub-Epic**: `Phase 0 — Governance, Repo, CI, Task Execution / Evidence & Audit Trail / CLI surface` (Tasks T-00531 .. T-00540).
- **Status**: 10/10 tasks completed.
- **Next Sub-Epic**: `Phase 0 — Governance, Repo, CI, Task Execution / Evidence & Audit Trail / MCP / API surface` starting at `T-00541`.
