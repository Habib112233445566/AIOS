# T-00444 — Documentation Index Control / MCP/API surface: Implementation

## 1. Implementation Scope
This task implements the MCP tool handlers for Documentation Index Control in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. Tool Implementations
- **`aios.doc.index.get`**:
  - Ingests `repo_path` (defaulting to `.`).
  - Calls `aiosh_core::doc_index_service::build_doc_index_from_paths`.
  - Emits `aios.doc.index.get` audit entry and returns JSON manifest.
- **`aios.doc.check`**:
  - Validates in-tree markdown links across documentation catalog.
  - Returns `ok: true` when all links are valid, `ok: false` with details when broken links are found.
  - Emits `aios.doc.check` audit entry.
- **`aios.doc.search`**:
  - Filters indexed entries for `query` match across title, path, and section.
  - Enforces mandatory `query` argument.
  - Emits `aios.doc.search` audit entry.

## 3. Test Verification
```text
running 2 tests
test tests::test_toolchain_tools_in_manifest ... ok
test tests::test_mcp_doc_tools_execution ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```
