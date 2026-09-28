# Task Evidence: T-02339 (Audit Chain Extensions / MCP/API surface: Documentation)

## 1. Metadata
- **Task ID:** `T-02339`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions MCP/API Surface Documentation
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 4: MCP/API Surface (9/10) — Documentation

---

## 2. Documentation Updates

Updated `code/aiosh-mcp/README.md` with comprehensive operator and agent reference documentation for the 4 newly exposed Audit Chain Extensions MCP tools:
- `aios.audit.query`
- `aios.audit.inspect`
- `aios.audit.ancestry`
- `aios.audit.sign_verify`

Included:
- Parameter reference table and required argument signatures.
- Copy-pasteable JSON-RPC 2.0 tool invocation example.
- Explicit constraints: max query limit of 1,000 rows (`MAX_QUERY_LIMIT = 1000`) and max ancestry traversal depth of 64 levels (`MAX_LINEAGE_DEPTH = 64`) with cyclic loop detection.
- Cross-references to task evidence files `T-02331` through `T-02338`.

---

## 3. Acceptance Confirmation
- [x] Documentation updated with working JSON-RPC invocation examples.
- [x] System constraints and boundary limits explicitly stated.
- [x] Evidence files cross-linked.
