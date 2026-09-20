# Task Evidence: T-01888 - Network Bootstrap / documentation: Hardening

## 1. Overview
- **Task ID**: `T-01888`
- **Sub-Epic**: 9 (Network Bootstrap Documentation Subsystem)
- **Goal**: Implement security hardening controls for `network_doc.rs` based on findings from `T-01887`.

---

## 2. Implemented Hardening Controls
1. **UTF-8 Character Boundary Safe Truncation**:
   - Replaced direct byte slicing (`&topic.summary[..117]`) with `chars().take(117).collect::<String>()`.
   - Guaranteed zero panic vectors when indexing or truncating summaries containing 3-byte or 4-byte UTF-8 sequences (emojis, Asian characters, special symbols).
2. **Search Query Token Bounds**:
   - Bounded `query_terms` extraction to a maximum of 16 whitespace-separated tokens via `query_terms.truncate(16)`, preventing CPU spinning on excessively long queries.
3. **Table Cell Sanitization (`sanitize_table_cell`)**:
   - Implemented `sanitize_table_cell` which strips control characters (`c.is_control() || c == '\0'`) and escapes Markdown table delimiters (`|` -> `\|`).
   - Applied sanitization to all dynamic fields in `render_state_markdown` (interface names, types, operstate, MACs, IP addresses, route destinations, gateways).
4. **Hardening Verification Suite**:
   - Authored `test_ndoc_hardening_features` verifying UTF-8 multi-byte emoji truncation, table cell pipe escaping, control character elimination, and 30-token query bounding.

---

## 3. Test Execution Verification
Command: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_network_doc`

```text
running 11 tests
test test_ndoc1_canonical_repository ... ok
test test_ndoc2_category_filtering ... ok
test test_ndoc2_category_loose_matching ... ok
test test_ndoc3_search_empty_and_bounded ... ok
test test_ndoc3_search_ranking ... ok
test test_ndoc4_render_topic_markdown ... ok
test test_ndoc5_render_ascii_topology ... ok
test test_ndoc5_render_state_markdown ... ok
test test_ndoc6_oversized_document_rejected ... ok
test test_ndoc_hardening_features ... ok
test test_ndoc6_persistence_atomic_and_path_hygiene ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```
Status: Verified & Passed (11/11 tests passing, zero warnings).
