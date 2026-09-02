# T-00598 — Evidence & Audit Trail / documentation: Hardening

## 1. Hardening Scope
This task verifies the structural hardening and mechanical rot-prevention invariants governing Evidence & Audit Trail documentation in `docs/README.md`.

## 2. Hardening Measures
- **Mechanical Invariant Enforcement (`tools/check_task_docs.py`)**:
  - `C1 (spec-health)`: Asserts valid specification headers and links.
  - `C2 (component sections)`: Asserts required component headers.
  - `C3 (referenced paths)`: Validates that all in-tree paths exist.
  - `C4 (phase map)`: Asserts Phase 0 task map stability.
  - `C5 (index health)`: Asserts spec index references.
  - `C6 (no volatile counts)`: Asserts static stability across tasks.
- **Security Policy Invariants (`tools/check_security_policy.py`)**:
  - Validates `S1`..`S5` against OpenSSF Scorecard criteria.

## 3. Verification Output
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
```
