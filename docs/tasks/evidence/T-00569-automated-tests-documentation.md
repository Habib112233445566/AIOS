# T-00569 — Evidence & Audit Trail / automated tests: Documentation

## 1. Documentation Scope
This task documents the automated test infrastructure and CI integration for Evidence & Audit Trail in `docs/README.md`.

## 2. Documentation Updates
- Updated `docs/README.md` under the Evidence & Audit Trail section with:
  - Invariant criteria `E1` (directory health), `E2` (ledger consistency), `E3` (file bounds), `E4` (hash consistency).
  - Standalone operator execution commands (`python tools/check_evidence.py` and `python tools/test_check_evidence.py`).
  - List of registered CI runner suites (`evidence_cli_smoke`, `evidence_mcp_smoke`, `evidence_checker`, `evidence_unit`).
  - Stated constraints and limitations (16 MiB size cap, bounded sampling).
  - Traceable evidence link range: `tasks/evidence/T-00511-data-model-research.md` .. `tasks/evidence/T-00569-automated-tests-documentation.md`.

## 3. Structural Validation
- Verified all structural documentation invariants `C1`..`C6` with `tools/check_task_docs.py`.

## 4. Verification Output
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```
