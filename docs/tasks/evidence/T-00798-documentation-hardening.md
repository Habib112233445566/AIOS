# T-00798 — Secrets & Access Hygiene / documentation: Hardening

## 1. Hardening Deliverables
- **Rot-Proof Invariant Enforcement**: Automated check `tools/check_task_docs.py` enforces that all documentation links and references resolve without broken paths.
- **Volatile Count Prohibition**: Living documentation never embeds hardcoded volatile count snapshots (criterion C6), ensuring docs remain accurate across subsequent task completions.
- **Fail-Closed Gate**: Documentation validation failure halts CI immediately.
