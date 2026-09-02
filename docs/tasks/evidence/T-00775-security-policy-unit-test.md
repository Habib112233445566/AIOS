# T-00775 — Secrets & Access Hygiene / security policy: Unit Test

## 1. Unit Test Deliverables
- Validated `tools/check_security_policy.py` verifying criteria S1..S5.
- Verified in-tree reference `docs/tasks/evidence/T-00777-security.md` resolves successfully under criterion S5.

## 2. Test Execution Output
```text
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
```
