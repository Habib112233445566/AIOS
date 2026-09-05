# T-01233: Package Management - MCP/API Surface: Scaffold

## Metadata
- **Task ID:** `T-01233`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Package Management MCP/API Surface Scaffold
- **Status:** Complete

## 1. Scaffold Deliverables
Scaffolded schemas and execution handlers in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
1. **Tool Schema Registration (`Server::tool_manifest`)**:
   - Registered `aios.package.search`:
     - Parameters: `pattern` (string, required), `limit` (integer, optional), `store_path` (string, optional), `grant_id` (string, optional).
   - Registered `aios.package.apply`:
     - Parameters: `actions` (array of objects, optional), `plan` (object, optional), `dry_run` (boolean, optional), `store_path` (string, optional), `grant_id` (string, optional).
2. **Tool Dispatch Wiring (`Server::call_tool`)**:
   - `aios.package.search`:
     - Validates `pattern` presence, 256-character length ceiling, and control-character prohibition.
     - Validates `limit` bounds [1..10,000].
     - Loads store (`load_from_path` or default) and runs `store.query(...)`.
     - Routes through `dispatch::recorded_call` with `is_write: false`.
   - `aios.package.apply`:
     - Skeleton handler validating parameter presence (`actions` or `plan`), with `is_write: true` for state-changing transactions.
     - Returns scaffold stub error `NotImplementedError` awaiting implementation in `T-01234`.

## 2. Compilation & Unit Test Verification
Executed targeted test runner:
```bash
cargo test --manifest-path code/aiosh-rust/Cargo.toml --bin aiosh-mcp test_mcp_package_tools
```
Output:
```text
running 1 test
test tests::test_mcp_package_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.07s
```
Both `aios.package.search` and `aios.package.apply` are discoverable via `tool_manifest` and cleanly dispatched in `call_tool`.
