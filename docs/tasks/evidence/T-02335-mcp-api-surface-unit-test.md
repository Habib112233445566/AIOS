# Task Evidence: T-02335 (Audit Chain Extensions / MCP/API surface: Unit Test)

## 1. Metadata
- **Task ID:** `T-02335`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions MCP/API Surface Unit Test (`code/aiosh-mcp/tests/test_audit_chain_mcp.py`)
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 4: MCP/API Surface (5/10) — Unit Test

---

## 2. Test Scope & Invariants Covered

Created dedicated, standalone test suite in `code/aiosh-mcp/tests/test_audit_chain_mcp.py` exercising all 4 Audit Chain Extensions tools over MCP JSON-RPC 2.0 stdio:
1. **Tool Registration & Schema Invariants (`test_mcp_audit_chain_tool_registration`)**:
   - Confirms presence of `aios.audit.query`, `aios.audit.inspect`, `aios.audit.ancestry`, and `aios.audit.sign_verify` in `tools/list`.
   - Validates input schema constraints (required fields `hash` on `inspect`, `ancestry`, `sign_verify`).
2. **Query Filtering & Bounds Enforcing (`test_mcp_audit_query_filtering_and_bounds`)**:
   - Validates unfiltered query pagination limits.
   - Tests filtering by actor and tool name.
   - Tests boundary conditions with non-matching filters returning clean empty match arrays.
3. **Event Inspection & Error Handling (`test_mcp_audit_inspect_valid_and_missing`)**:
   - Asserts failure on missing required `hash` parameter.
   - Asserts failure on non-existent hash (`ok: false` with not found error).
   - Validates successful retrieval and structure of event record on known hash.
4. **Causal Ancestry & Signature Verification (`test_mcp_audit_ancestry_and_sign_verify`)**:
   - Asserts error handling on missing parameters.
   - Validates ancestry DAG traversal with depth limits.
   - Validates Ed25519 digital signature verification report format and invariant reporting (`has_signature`, `is_valid`).

---

## 3. Test Execution & Output

```text
> python code/aiosh-mcp/tests/test_audit_chain_mcp.py
Running test_mcp_audit_chain_tool_registration...
Running test_mcp_audit_query_filtering_and_bounds...
Running test_mcp_audit_inspect_valid_and_missing...
Running test_mcp_audit_ancestry_and_sign_verify...
=== All Audit Chain MCP Unit Tests Passed Successfully ===
```

---

## 4. Acceptance Confirmation
- [x] Dedicated unit test file `code/aiosh-mcp/tests/test_audit_chain_mcp.py` authored and runs standalone.
- [x] Positive paths, boundary values, and negative error modes asserted.
- [x] Clean zero-exit pass achieved.
