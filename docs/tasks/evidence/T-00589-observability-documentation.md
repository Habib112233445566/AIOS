# T-00589 — Evidence & Audit Trail / observability: Documentation

## 1. Documentation Scope
This task documents the observability architecture, telemetry models (`EvidenceTelemetry`), and verification diagnostic workflows in `docs/README.md`.

## 2. Documentation Deliverables
- Updated `docs/README.md` with:
  - Specification of the `EvidenceTelemetry` schema (`total_records`, `valid_records`, `missing_files_count`, `hash_mismatches_count`, `is_healthy`).
  - Diagnostic invocation examples using `aiosh evidence verify --json`.
  - Copy-pasteable telemetry JSON response example.
  - Clarification of 512-byte outcome string clamping (`clamp_str`) on audit log entries.
  - Updated evidence reference range: `tasks/evidence/T-00511-data-model-research.md` .. `tasks/evidence/T-00589-observability-documentation.md`.

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
