# Task Evidence: T-02336 (Audit Chain Extensions / MCP/API surface: Integration)

## 1. Metadata
- **Task ID:** `T-02336`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions MCP/API Surface Integration
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 4: MCP/API Surface (6/10) — Integration

---

## 2. Integration Architecture & Cross-Substrate Parity

1. **Production Wire-up**:
   - The Audit Chain Extensions MCP tools (`aios.audit.query`, `aios.audit.inspect`, `aios.audit.ancestry`, `aios.audit.sign_verify`) are registered in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
   - Dispatch paths wire into `aiosh_core::audit_chain_service::AuditChainService` backed by the shared SQLite `AuditRing`.
   - Every tool call executes within PEP fabric governance via `dispatch::recorded_call`, writing an immutable audit record of its own invocation into the chain.

2. **Cross-Substrate Parity**:
   - Shared SQLite DB schema with extended columns (`provenance_json`, `causal_links_json`, `signature_json`, `extensions_json`) is accessed uniformly across CLI (`aiosh audit`), MCP daemon (`aiosh-mcp`), and Core Rust libraries (`AuditRing`).
   - Canonical JSON hash generation ensures exact cryptographic parity regardless of whether the record was inserted via Rust core, CLI commands, or MCP tools.

---

## 3. Verification Output

```text
> python -m pytest code/aiosh-mcp/tests/test_audit_chain_mcp.py
============================= test session starts =============================
platform win32 -- Python 3.14.6, pytest-9.1.1, pluggy-1.6.0
rootdir: C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-mcp
configfile: pyproject.toml
plugins: anyio-4.14.2
collected 4 items

code\aiosh-mcp\tests\test_audit_chain_mcp.py ....                        [100%]

============================== 4 passed in 0.88s ==============================
```

---

## 4. Acceptance Confirmation
- [x] Feature reachable through its production MCP JSON-RPC 2.0 surface.
- [x] Integration smoke tests pass end-to-end.
- [x] Cross-substrate DB schema and hash verification parity validated.
