# T-02284 Implementation: Grant Lifecycle Documentation

**Task:** Implement the minimal working behavior for the documentation of Grant Lifecycle.  
**Status:** COMPLETE  
**Date:** 2026-09-22  
**Epic:** Phase 2 — Security Kernel & PEP Fabric  
**Sub-Epic:** Grant Lifecycle / Documentation  

---

## 1. What Was Implemented

1. **`PepGrantDocIndex` Core Search & Retrieval Engine:**
   - Pre-populated repository of 7 canonical grant lifecycle topics:
     - `grant-arch`: Architecture, data models, scopes, constraints.
     - `grant-lifecycle`: State machine (Requested, Active, Suspended, Revoked, Expired).
     - `grant-attenuation`: Monotonicity invariants, depth decrement, delegation.
     - `grant-revocation`: Targeted and cascading tree revocation, expiration sweeps.
     - `grant-policy`: `PepGrantSecurityPolicy`, mode governance, duration bounds.
     - `grant-observability`: Telemetry aggregation and health thresholds.
     - `grant-mcp`: MCP tool interfaces and JSON-RPC parameter schemas.

2. **Scoring & Lexical Search:**
   - Exact tag matches score +10 points.
   - Title matches score +5 points.
   - Summary matches score +3 points.
   - Section body matches score +1 point.
   - Snippet extraction truncated cleanly at 200 characters (`MAX_GRANT_DOC_SNIPPET_LEN`).
   - Query length bounded to 128 characters (`MAX_GRANT_DOC_QUERY_LEN`).

3. **Markdown Formatter:**
   - `render_markdown()` converts topics into GitHub-flavored Markdown.

---

## 2. Acceptance Verification
- ✅ Complete working documentation engine with 7 canonical topics.
- ✅ Bounds-checked search logic with weighted relevance scoring.
- ✅ Clean compilation across `aiosh-core`.
