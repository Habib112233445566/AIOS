# Integration Summary: T-02086 (documentation: Integration)

- **Target Component**: `code/aiosh-rust/aiosh-mcp/src/main.rs`
- **Tool Added**: `aios.capability.doc`
- **Capabilities**:
  - `action: "list"`: enumerates all topics or filters by `category`.
  - `action: "get"`: fetches topic details by `topic_id` with rendered Markdown.
  - `action: "search"`: relevance-scored search with UTF-8 safe snippets.
- **Verification**:
  - Tested in `test_capability_mcp_tools` unit test.
  - Verified via `code/aiosh-mcp/tests/test_capability_doc_smoke.py`.
