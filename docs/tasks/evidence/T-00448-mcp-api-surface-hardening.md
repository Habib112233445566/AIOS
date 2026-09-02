# T-00448 — Documentation Index Control / MCP/API surface: Hardening

## 1. Hardening Scope
This task verifies and documents hardening protections for the Documentation Index Control MCP tools in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. Hardening Measures
1. **Structured Error Propagation**:
   - Missing fields, document parsing errors, and invalid repo paths return structured error envelopes (`{"ok": false, "tool": "...", "error": "..."}`) rather than panicking or aborting the MCP stdio loop.
2. **Honest Audit Logging on Failure**:
   - `dispatch::recorded_call` emits audit rows on both success (`outcome: "ok"`) and error (`outcome: "error"`), recording failure reasons in `outcome_detail`.
3. **Resource Bound Guarantees**:
   - Document loading bounded by 16 MiB per file.
   - All allocations and database transactions are contained within single request lifecycles.

## 3. Verification
- Negative testing in `test_doc_mcp_smoke.py` confirms that invalid invocations produce proper error responses without crashing the server daemon.
