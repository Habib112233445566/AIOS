# T-00476 — Documentation Index Control / security policy: Integration

## 1. Integration Scope
This task integrates the Documentation Index Control security policy with PEP token enforcement in `code/aiosh-rust/aiosh-core/src/pep.rs`, core service enforcement in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs`, and repository governance in `SECURITY.md`.

## 2. Integrated Components
1. **PEP Subsystem (`code/aiosh-rust/aiosh-core/src/pep.rs`)**:
   - `aios.doc.set` and `doc.set` added to `is_irreversible` matcher.
   - Mutating commands require active verified PEP grant tokens.
2. **Core Service Policy (`code/aiosh-rust/aiosh-core/src/doc_index_service.rs`)**:
   - `check_doc_index_policy()` enforces PEP gating for irreversible tools.
3. **Repository Security Policy (`SECURITY.md`)**:
   - Added vulnerability clauses covering documentation directory traversal and resource exhaustion via uncapped document ingestion.
   - Linked `T-00467-security.md` and `T-00477-security.md` in the Security Knowledge Index.

## 3. Verification
- `cargo test -p aiosh-core test_check_doc_index_policy_enforcement` -> PASS
- `python tools/check_security_policy.py` -> PASS (S1..S5)
- All integrated policy enforcement paths verified green.
