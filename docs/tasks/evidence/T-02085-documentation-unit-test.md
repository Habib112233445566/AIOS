# Task Evidence: T-02085 (documentation: Unit Test)

## Overview
- **Task ID**: T-02085
- **Sub-Epic**: Sub-Epic 9: Capability Documentation Subsystem
- **Component**: `aiosh-core::capability_doc`
- **Objective**: Author and execute comprehensive unit tests for `CapabilityDocIndex`, validating canonical topics, deterministic lookup, category filtering, search ranking, defensive bounds, UTF-8 safe snippet extraction, and serialization.

## Unit Test Matrix (`tests/test_capability_doc.rs`)

| Test Function | Invariant | Description |
|---|---|---|
| `test_doc_index_canonical_topics_present` | `CAPDOC1` | Validates presence and non-empty fields for all 8 canonical capability topics. |
| `test_doc_index_get_topic` | `CAPDOC2`, `CAPDOC5` | Validates case-insensitive ID retrieval, whitespace trimming, non-existent handling, and rejection of oversized or control-character IDs. |
| `test_doc_index_list_by_category` | `CAPDOC1` | Validates partition filtering across Architecture, Lifecycle, Security, Observability, and Reference categories. |
| `test_doc_index_search_scoring` | `CAPDOC3` | Validates ranking hierarchy (exact ID >= 100, tag >= 50, title >= 25, summary >= 15, content >= 10) and descending sort order. |
| `test_doc_index_search_defensive_bounds` | `CAPDOC5` | Validates rejection of empty queries, control-character queries, and queries exceeding `MAX_DOC_QUERY_LEN` (256 chars). |
| `test_doc_index_utf8_snippet_safety` | `CAPDOC4` | Validates safe snippet extraction without panic or byte-slicing errors across multi-byte UTF-8 inputs (emojis, CJK, accents). |
| `test_doc_index_markdown_formatting` | `CAPDOC6` | Validates structured Markdown generation with headings, tags, examples, and references. |
| `test_doc_index_serde` | `CAPDOC6` | Validates round-trip JSON serialization and deserialization for topics and search results. |

## Verification
- Executed via `cargo test --test test_capability_doc`.
