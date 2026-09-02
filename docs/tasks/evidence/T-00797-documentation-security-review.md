# T-00797 — Secrets & Access Hygiene / documentation: Security Review

## 1. Threat Model & Documentation Security Review

| Scenario ID | Attack Vector | Mitigation / Verification | Status |
|---|---|---|---|
| **DOC-SEC-1** | Real Secret Leakage in Example Code | All example commands, JSON requests, and schema illustrations use sanitized dummy strings (`AKIA...`, `ghp_...`) conforming to public sample standards. | Mitigated |
| **DOC-SEC-2** | Phishing / SSRF Link Injection | `tools/check_task_docs.py` (criterion C3 and C5) ensures all backticked paths and markdown links resolve strictly to in-tree files inside the repository checkout. | Mitigated |
| **DOC-SEC-3** | Inaccurate Security Claims | Documented capabilities match actual implementations in `aiosh-core`, `aiosh-cli`, and `aiosh-mcp`. | Mitigated |

## 2. Policy Invariants
- Zero live secrets in markdown or comments.
- All in-tree links are verified.
