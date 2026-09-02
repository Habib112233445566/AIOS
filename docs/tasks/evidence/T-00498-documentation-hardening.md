# T-00498 — Documentation Index Control / documentation: Hardening

## 1. Hardening Overview
This task hardens the documentation formatting and reporting helpers for Documentation Index Control against unexpected input bounds, empty catalogs, and formatting errors.

## 2. Hardening Measures
1. **Empty Manifest Resilience**:
   - `format_doc_index_summary` cleanly renders `(no documents indexed)` rather than producing empty output or panicking when no entries exist.
2. **Schema Invariant Enforcement**:
   - `DocIndexManifest::validate()` ensures version non-emptiness and rejects malformed entries before string formatting occurs.
3. **CI Invariant Guarding**:
   - `tools/check_task_docs.py` enforces C1..C6 structural invariants on all documentation references.
4. **Honest Audit Emission**:
   - Every CLI doc query logs structured event outcomes to SQLite WAL.

## 3. Verification
- `cargo test -p aiosh-core test_format_doc_index_summary` -> 3/3 passed.
- `python tools/check_task_docs.py` -> PASS (C1..C6).
- `python tools/test_doc_index_suites.py` -> PASS (D1..D7).
