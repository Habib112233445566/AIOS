# T-00609 — Evidence & Audit Trail / recovery & validation: Documentation

## 1. Documentation Scope
This task documents the recovery helpers (`recover_default_evidence_config`, `reconstruct_evidence_manifest`, and `reconcile_evidence_manifest`) in `docs/README.md` and extends the evidence link range through `T-00609-recovery-validation-documentation.md`.

## 2. Documentation Deliverables
- Updated `docs/README.md` with:
  - Specification of `recover_default_evidence_config()` for canonical configuration fallback.
  - Specification of `reconstruct_evidence_manifest(&repo, range, epic)` for live disk manifest reconstruction.
  - Specification of `reconcile_evidence_manifest(&repo, &manifest)` for verification and aggregate telemetry generation.
  - Updated evidence reference range: `tasks/evidence/T-00511-data-model-research.md` .. `tasks/evidence/T-00609-recovery-validation-documentation.md`.

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
