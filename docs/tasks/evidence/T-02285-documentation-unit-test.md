# T-02285 Unit Test: Grant Lifecycle Documentation

**Task:** Add focused automated tests for the documentation of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Documentation  

---

## 1. Unit Tests Created

Created `code/aiosh-rust/aiosh-core/tests/test_pep_grant_doc.rs` with 5 focused test functions:

| Test Name | Vector Tested | Behavior Asserted |
|---|---|---|
| `test_grant_doc_index_canonical_topics_count` | Inventory completeness | Confirms exactly 7 canonical topics exist with expected topic IDs |
| `test_grant_doc_get_topic_positive_and_negative` | ID lookup & negative handling | Retrieves valid topic (`grant-arch`), returns None on missing topic |
| `test_grant_doc_search_relevance` | Lexical search ranking | Searches for 'attenuation' and 'cascade', confirms top-ranked match accuracy |
| `test_grant_doc_search_bounds` | Input boundary constraints | Rejects empty and overlong (> 128 chars) queries with `GRANTDOC_ERR_QUERY_BOUNDS` |
| `test_grant_doc_render_markdown` | Document rendering | Validates header, category, metadata, and example generation in Markdown |

---

## 2. Acceptance Verification
- ✅ Standalone test file created under `tests/`.
- ✅ All 5 tests pass asserting observable documentation behavior and query bounds.
- ✅ Negative error cases (`GRANTDOC_ERR_QUERY_BOUNDS`, `GRANTDOC_ERR_NOT_FOUND`) explicitly verified.
