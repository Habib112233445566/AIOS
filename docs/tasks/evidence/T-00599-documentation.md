# T-00599 — Evidence & Audit Trail / documentation: Documentation

## 1. Documentation Scope
This task updates `docs/README.md` to document human-readable manifest text formatting (`format_evidence_summary`) and extends the evidence link range through `T-00599-documentation-documentation.md`.

## 2. Documentation Deliverables
- Updated `docs/README.md` with:
  - Documented `format_evidence_summary(&manifest)` helper for human-readable manifest console output.
  - Updated evidence reference range: `tasks/evidence/T-00511-data-model-research.md` .. `tasks/evidence/T-00599-documentation-documentation.md`.

## 3. Structural Validation
- `tools/check_task_docs.py` -> C1..C6 PASS.
- `tools/check_security_policy.py` -> S1..S5 PASS.

## 4. Verification Output
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
