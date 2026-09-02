# T-00479 — Documentation Index Control / security policy: Documentation

## 1. Documentation Scope
This task updates user and operator documentation in `docs/README.md` and repository security policy in `SECURITY.md` regarding the security policy, PEP token gating, and CI verification rules for Documentation Index Control.

## 2. Documentation Updates
1. **`docs/README.md`**:
   - Added **Security Policy & PEP** section under Documentation Index Control detailing `check_doc_index_policy()`, unauthenticated read-only queries, token gating for mutating actions (`aios.doc.set`, `doc.set`), bounded memory caps (16 MiB file read / 64 KiB config read), and repository root link containment.
   - Updated evidence chain range (`T-00411`..`T-00478`).
2. **`SECURITY.md`**:
   - Explicitly listed path traversal escaping checkout bounds and denial-of-service via uncapped document ingestion under "What Counts as a Vulnerability".
   - Linked `T-00467-security.md` and `T-00477-security.md` in the Security Knowledge Index.

## 3. Verification
- `python tools/check_task_docs.py` -> PASS (C1..C6)
- `python tools/check_security_policy.py` -> PASS (S1..S5)
