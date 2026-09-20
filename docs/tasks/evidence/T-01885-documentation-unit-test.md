# Task Evidence: T-01885 - Network Bootstrap / documentation: Unit Test

## 1. Overview
- **Task ID**: `T-01885`
- **Sub-Epic**: 9 (Network Bootstrap Documentation Subsystem)
- **Goal**: Author and execute comprehensive unit tests for Network Bootstrap Documentation in `code/aiosh-rust/aiosh-core/tests/test_network_doc.rs`.

---

## 2. Test Cases & Coverage
1. `test_ndoc1_canonical_repository`: Validates repository instantiation, presence of all 6 canonical topics covering architecture, discovery, security policy, observability, configuration, and troubleshooting.
2. `test_ndoc2_category_loose_matching`: Validates loose string matching of categories with case-insensitivity and prefix/substring matching (`arch`, `probe`, `sec`, `obs`, `cfg`, `triage`).
3. `test_ndoc2_category_filtering`: Validates topic filtering by category, ensuring correct topics are retrieved per category filter.
4. `test_ndoc3_search_ranking`: Validates multi-field relevance scoring (ID match +100, Title match +50, Tag match +25, Summary match +20, Section match +5), verifying exact topic matching ranks highest.
5. `test_ndoc3_search_empty_and_bounded`: Validates empty query returns empty results and queries are bounded to `MAX_SEARCH_RESULTS` (20).
6. `test_ndoc4_render_topic_markdown`: Validates Markdown formatting of reference topics, checking metadata headers, RFC citations, copy-pasteable configuration/command blocks, and related topic links.
7. `test_ndoc5_render_state_markdown`: Validates dynamic Markdown generation from network state, including interface tables, routing tables, DNS resolvers, and status indicators.
8. `test_ndoc5_render_ascii_topology`: Validates generation of ASCII topology tree diagram showing host, default gateway, interfaces, and configured routes.
9. `test_ndoc6_persistence_atomic_and_path_hygiene`: Validates JSON serialization, roundtrip deserialization, atomic write via sibling temp file, and path hygiene checks (`..` and control characters rejected).
10. `test_ndoc6_oversized_document_rejected`: Validates rejection of oversized documents exceeding `MAX_DOC_FILE_BYTES` (1 MB).

---

## 3. Test Execution Verification
Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_network_doc`

```text
running 10 tests
test test_ndoc1_canonical_repository ... ok
test test_ndoc2_category_loose_matching ... ok
test test_ndoc2_category_filtering ... ok
test test_ndoc3_search_empty_and_bounded ... ok
test test_ndoc3_search_ranking ... ok
test test_ndoc4_render_topic_markdown ... ok
test test_ndoc5_render_ascii_topology ... ok
test test_ndoc5_render_state_markdown ... ok
test test_ndoc6_oversized_document_rejected ... ok
test test_ndoc6_persistence_atomic_and_path_hygiene ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```
Status: PASS (10/10 passed, 0 failures, 0 warnings).
