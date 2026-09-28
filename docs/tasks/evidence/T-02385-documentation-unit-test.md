# Task Evidence: T-02385 - Audit Chain Extensions: Documentation Unit Test

## Goal
Add focused automated tests for the documentation subsystem of Audit Chain Extensions.

## Test Coverage
Created `code/aiosh-rust/aiosh-core/tests/test_audit_chain_doc.rs` exercising:
1. `test_audit_doc_index_canonical_topics_count`: Validates that all 7 canonical topics are registered (`audit-arch`, `audit-lineage`, `audit-crypto`, `audit-policy`, `audit-observability`, `audit-recovery`, `audit-reference`).
2. `test_audit_doc_get_topic_positive_and_negative`: Validates positive retrieval of topics and assertions on None for non-existent IDs.
3. `test_audit_doc_search_relevance`: Validates keyword scoring and snippet generation for terms like "lineage" and "ed25519".
4. `test_audit_doc_search_bounds`: Tests empty query rejection and bounds enforcement against queries exceeding `MAX_AUDIT_DOC_QUERY_LEN`.
5. `test_audit_doc_render_markdown`: Asserts Markdown rendering format and error handling for missing topics.
6. `test_audit_doc_category_filter`: Tests category filtering for `Observability` and `Recovery`.

## Test Execution Results
```
running 6 tests
test test_audit_doc_index_canonical_topics_count ... ok
test test_audit_doc_get_topic_positive_and_negative ... ok
test test_audit_doc_category_filter ... ok
test test_audit_doc_render_markdown ... ok
test test_audit_doc_search_bounds ... ok
test test_audit_doc_search_relevance ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```
All 6 tests passed standalone.
