# Implementation Summary: T-02084 (documentation: Implementation)

- **Target File**: `code/aiosh-rust/aiosh-core/src/capability_doc.rs`
- **Features Implemented**:
  - In-memory index `CapabilityDocIndex` with 8 pre-registered canonical topics covering `CAP1..CAP6`, `CAPSEC1..CAPSEC6`, and `CAPOBS1..CAPOBS6`.
  - Scored search engine with defensive bounds and deterministic ordering.
  - Safe UTF-8 snippet extractor (`extract_utf8_snippet`).
  - Category filtering, ID lookup, and Markdown formatter.
- **Status**: Compiles cleanly with `aiosh-core`.
