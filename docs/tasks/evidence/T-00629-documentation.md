# T-00629 — Repository Health / core service: Documentation

## 1. Documentation Scope
This task documents the core service operations of the **Repository Health** component (`T-00611..T-00710`) in `docs/README.md`.

## 2. Documentation Deliverables
- Documented `check_git_working_tree`, `check_file_bounds`, `check_security_governance`, and `check_repo_health`.
- Added copy-pasteable Rust orchestrator code example.
- Updated evidence range: `tasks/evidence/T-00611-data-model-research.md` .. `tasks/evidence/T-00629-core-service-documentation.md`.

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
