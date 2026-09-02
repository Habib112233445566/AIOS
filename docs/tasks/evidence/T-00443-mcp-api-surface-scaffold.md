# T-00443 — Documentation Index Control / MCP/API surface: Scaffold

## 1. Scaffold Scope
This task registers the Documentation Index Control tool schemas in `code/aiosh-rust/aiosh-mcp/src/main.rs` and defines scaffold dispatch handlers verified by `#[should_panic]` test stubs.

## 2. Scaffold Registrations
- In `Server::list_tools()`:
  - `aios.doc.index.get`: Parameter schema for retrieving documentation index manifest.
  - `aios.doc.check`: Parameter schema for running in-tree link validation.
  - `aios.doc.search`: Parameter schema requiring query string for catalog search.
- In `Server::call_tool()`:
  - Added skeleton arm for `aios.doc.*` throwing `unimplemented!("T-00443: aiosh-mcp doc scaffold")`.

## 3. Test Verification
```text
running 2 tests
test tests::test_toolchain_tools_in_manifest ... ok
test tests::test_mcp_doc_scaffold - should panic ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
```
