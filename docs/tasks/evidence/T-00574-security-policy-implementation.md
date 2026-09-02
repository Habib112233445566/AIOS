# T-00574 — Evidence & Audit Trail / security policy: Implementation

## 1. Implementation Scope
This task implements the security policy updates for Evidence & Audit Trail in root `SECURITY.md`.

## 2. Policy Deliverables
- Updated `SECURITY.md` under **What Counts as a Vulnerability** with:
  - Falsifying, forging, or tampering with SHA-256 evidence digests, historical task completion artifacts, or provenance logs.
  - Path traversal escapes or out-of-bounds filesystem discovery during evidence scanning and verification.
- Updated **Security Knowledge Index** in `SECURITY.md` with:
  - `docs/tasks/evidence/T-00567-security.md` (evidence automated tests)
  - `docs/tasks/evidence/T-00577-security.md` (evidence security policy)

## 3. Policy Criteria Verification
- Validated via `tools/check_security_policy.py`:
  - `S1`: `SECURITY.md` exists and contains no unresolved TODOs.
  - `S2`: Private advisory reporting URL is present verbatim.
  - `S3`: Free-form prose floor (>1200 characters).
  - `S4`: Standard policy terminology hits (`vuln=3`, `disclos=3`, `day-count=True`).
  - `S5`: All referenced in-tree paths exist and resolve accurately.

## 4. Verification Output
```text
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
```
