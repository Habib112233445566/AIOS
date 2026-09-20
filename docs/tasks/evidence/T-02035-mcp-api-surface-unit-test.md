# T-02035: Capability Model / MCP/API Surface — Unit Test

**Task ID**: `T-02035`  
**Phase**: Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic**: Sub-Epic 4: Capability Model / MCP/API Surface  
**Status**: COMPLETED  
**Date**: 2026-09-20  

---

## 1. Test Suite Summary

A comprehensive automated unit test suite `test_capability_mcp_tools` was implemented in `code/aiosh-rust/aiosh-mcp/src/main.rs`:
- **File**: `code/aiosh-rust/aiosh-mcp/src/main.rs`
- **Test Target**: `test_capability_mcp_tools`
- **Result**: `1 passed; 0 failed; 0 ignored; finished in 0.08s`

---

## 2. Test Coverage

The unit test exercises all 7 MCP capability tools covering happy paths, negative security cases, boundary conditions, and state transitions:

1. **Manifest Registration**:
   - Asserts all 7 tools (`list`, `get`, `issue`, `attenuate`, `revoke`, `check`, `prune`) are published in `Server::tool_manifest`.
2. **`aios.capability.list`**:
   - Initial empty query returns `ok: true`, `count: 0`.
3. **`aios.capability.issue`**:
   - Negative: Unauthorized issuer (`untrusted:user`) rejected with `ok: false`.
   - Positive: Root issuance (`issuer: "kernel"`) succeeds, returns valid `CAP-<uuid>` ID.
4. **`aios.capability.get`**:
   - Retrieves capability by ID, verifies subject `agent:worker`.
5. **`aios.capability.attenuate`**:
   - Negative: Privilege escalation attempt (`rights: ["admin"]` when parent lacks it) rejected with `ok: false`.
   - Positive: Monotonic child capability created with subset rights (`["read"]`) and narrowed invocation quota.
6. **`aios.capability.check`**:
   - Positive: Granted check for authorized right (`read`), consumes 1 invocation, confirms remaining quota is 9.
   - Negative: Ungranted right (`write`) on child returns `granted: false`.
7. **`aios.capability.revoke`**:
   - Cascade revocation of root capability revokes both root and child IDs.
   - Subsequent `check` returns `granted: false`.
8. **`aios.capability.prune`**:
   - Pruning invocation succeeds and reports pruned count.

---

## 3. Test Execution Capture

```text
running 1 test
test tests::test_capability_mcp_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 0.08s
```
