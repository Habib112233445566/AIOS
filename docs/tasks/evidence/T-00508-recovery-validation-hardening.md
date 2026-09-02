# T-00508 — Documentation Index Control / recovery & validation: Hardening

## 1. Hardening Overview
This task hardens the recovery, validation, and reconciliation workflows of Documentation Index Control against resource exhaustion, malformed file paths, unbounded link traversals, and unhandled errors.

## 2. Hardening Measures
1. **Size Caps on Ingestion**:
   - File ingestion is capped at 16 MiB per document; configuration parsing is capped at 64 KiB.
2. **Deterministic Catalog Validation**:
   - `validate_doc_index_catalog` fails closed when broken links or traversal escapes are detected, emitting an explicit error with the exact broken link count.
3. **Multi-Document Reconciliation Error Envelopes**:
   - `reconcile_doc_index` halts with an explicit `Err` on missing files without partially processing broken state or leaking memory.
4. **Audit Invariants**:
   - All outcomes (success or error) emit structured audit logs to SQLite WAL.

## 3. Verification
- `cargo test -p aiosh-core test_validate_and_reconcile_doc_index_happy` -> PASS.
- `cargo test -p aiosh-core test_validate_doc_index_catalog_broken_link_error` -> PASS.
- `cargo test -p aiosh-core test_reconcile_doc_index_missing_file_error` -> PASS.
- `python tools/test_doc_index_suites.py` -> PASS (D1..D7).
