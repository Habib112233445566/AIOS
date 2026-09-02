# T-00839 — Regression Triage / CLI: Documentation

## 1. Documentation Review & Status
- Published Regression Triage CLI reference in `docs/README.md` under `## Regression Triage (T-00811..T-00910)`.
- Documented criteria T1..T3 in `tools/test_triage_suites.py`.
- Verified evidence chain `docs/tasks/evidence/T-00831-cli-research.md` .. `docs/tasks/evidence/T-00839-cli-documentation.md`.

## 2. Invariant Validation
- Ran `python tools/check_task_docs.py` (criteria C1..C6 PASS).
- Ran `python tools/check_security_policy.py` (criteria S1..S5 PASS).
- Ran `python tools/test_triage_suites.py` (criteria T1..T3 PASS).
