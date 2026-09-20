# Task Evidence: T-01985 - System Update / documentation: Unit Test (Sub-Epic 9)

## 1. Overview
- **Task ID**: `T-01985`
- **Sub-Epic**: 9 (System Update Documentation Subsystem)
- **Goal**: Implement focused automated unit tests in `code/aiosh-rust/aiosh-core/tests/test_system_update_doc.rs` verifying invariants `UDOC1..UDOC6`.

---

## 2. Test Suite Details
The unit test suite validates:
1. `test_udoc1_canonical_index_population`: Verifies that `SystemUpdateDocIndex::new()` initializes all 6 technical topics and accurately retrieves by unique ID.
2. `test_udoc2_category_navigation_and_loose_parsing`: Tests category filtering and loose string parsing across standard aliases (`arch`, `slots`, `policy`, `telemetry`, `config`, `debug`).
3. `test_udoc3_ranked_search`: Validates ranked multi-keyword full-text search, asserting that exact ID matches, tag matches, and title matches score appropriately, and handles empty queries safely.
4. `test_udoc4_markdown_export`: Verifies Markdown serialization for individual topics and complete index sections.
5. `test_udoc5_dynamic_rendering`: Tests dynamic Markdown report generation from live `SystemUpdateService` (including ASCII slot visualization) and `SystemUpdateObservabilityReport`.
6. `test_udoc6_file_export_and_hygiene`: Validates file export, directory creation, 1 MB bounds, and symlink attack rejection.
