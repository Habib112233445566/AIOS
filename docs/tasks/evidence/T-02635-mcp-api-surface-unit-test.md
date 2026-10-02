# Task Evidence: T-02635 — Secrets Handling MCP/API Surface Unit Test

## 1. Summary
Added comprehensive automated unit tests for the Secrets Handling MCP surface in `code/aiosh-rust/aiosh-mcp/src/main.rs` (`test_mcp_secret_tools_execution`):
1. **Tool Advertisement**:
   - Asserts all 5 secret tools (`aios.secret.store`, `aios.secret.get`, `aios.secret.list`, `aios.secret.rotate`, `aios.secret.revoke`) are registered in `Server::tool_manifest()`.
2. **Negative Tests & Path Traversal**:
   - Asserts traversal paths (`store_path: "../forbidden/vault.json"`) fail with `ok: false`.
   - Asserts missing required arguments in `aios.secret.store` fail with `ok: false`.
   - Asserts query for non-existent secret in `aios.secret.get` returns `ok: false`.
3. **Happy Path & Lifecycle**:
   - `aios.secret.store`: Successfully registers secret with custom store path.
   - `aios.secret.get` (default): Returns masked value (`exposed: false`, payload hidden).
   - `aios.secret.get` (`expose: true`): Returns plaintext secret payload.
   - `aios.secret.list`: Verifies secret count and confirms raw secret is not present in output.
   - `aios.secret.rotate`: Updates payload, increments version to 2, recalculates fingerprint.
   - `aios.secret.revoke`: Transitions secret to revoked state.
   - Post-revocation `aios.secret.get`: Denied with `ok: false`.

## 2. Test Execution Output
```
     Running unittests src\main.rs (target\debug\deps\aiosh_mcp-e34ec656bee882fe.exe)

running 1 test
test tests::test_mcp_secret_tools_execution ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.14s
```
