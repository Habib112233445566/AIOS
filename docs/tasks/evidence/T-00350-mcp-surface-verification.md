# T-00350 — Dependency & Toolchain Pinning / MCP/API surface: Verification & Evidence

## Test Results
```
test result: ok. 13 passed; 0 failed (aiosh-cli)
test result: ok. 92 passed; 0 failed (aiosh-core)
test result: ok. 1 passed; 0 failed (aiosh-mcp)
test result: ok. 0 passed; 0 failed (aiosh-sandbox)
test result: ok. 0 passed; 0 failed (doc-tests)
```
**Total: 106 tests, 0 failures.**

## Sub-Epic Complete
The MCP surface sub-epic (T-00341 through T-00350) is complete.

### Achievements:
- Added `aios.toolchain.check` to MCP `tool_manifest()`.
- Handled invocation routing in `call_tool()`, enforcing toolchain via `aiosh_core::toolchain_service::enforce_toolchain()`.
- Successfully gated the invocation through `dispatch::recorded_call()` for immutable audit logging.
- Hardcoded `ToolchainManifest::from_env()` to prevent agent-driven configuration overrides, enhancing security.
- Tested integration of the new tool over the JSON-RPC interface and verified syntax / test passing in Rust workspace.
