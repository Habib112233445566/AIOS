# Unit Test Summary: T-02085 (documentation: Unit Test)

- **Test Suite**: `code/aiosh-rust/aiosh-core/tests/test_capability_doc.rs`
- **Tests Implemented**: 8 unit tests covering `CAPDOC1..CAPDOC6`.
- **Key Assertions**:
  - 8 canonical topics present with non-empty fields.
  - Case-insensitive retrieval and defensive bounds on topic ID lookup.
  - Category filtering returns expected partition counts.
  - Search ranking accurately reflects weighted match scoring.
  - Defensive query bounds enforce maximum length and control character exclusion.
  - UTF-8 snippet extractor handles multi-byte UTF-8 boundaries without panic.
  - Markdown formatting and JSON serde fidelity verified.
